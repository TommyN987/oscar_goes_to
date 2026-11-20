//! The binary as a process, exporting to a fake OTLP collector:
//! what SIGTERM does to a request in flight, and what ties an exported log record to its trace.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, ExitStatus};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sqlx::postgres::PgConnectOptions;
use sqlx::{AssertSqlSafe, ConnectOptions, Connection, PgConnection};
use tokio::runtime::Runtime;

/// One OTLP export the collector received.
struct Export {
    path: String,
    body: Vec<u8>,
}

#[derive(Default)]
struct Hold {
    holding: bool,
    held_exports: usize,
}

/// An OTLP/HTTP collector that records every export and can be told to hold its answers back.
#[derive(Clone)]
struct FakeCollector {
    endpoint: String,
    exports: Arc<Mutex<Vec<Export>>>,
    hold: Arc<(Mutex<Hold>, Condvar)>,
}

impl FakeCollector {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let collector = Self {
            endpoint: format!("http://{}", listener.local_addr().unwrap()),
            exports: Arc::default(),
            hold: Arc::default(),
        };
        let accepting = collector.clone();
        thread::spawn(move || {
            for connection in listener.incoming().flatten() {
                let serving = accepting.clone();
                thread::spawn(move || serving.serve(connection));
            }
        });
        collector
    }

    /// Answers every export on one connection; an answer waits for as long as the collector is holding.
    fn serve(&self, connection: TcpStream) {
        let mut reader = BufReader::new(connection.try_clone().unwrap());
        let mut connection = connection;
        while let Some(export) = read_export(&mut reader) {
            self.exports.lock().unwrap().push(export);

            let (hold, changed) = &*self.hold;
            let mut hold = hold.lock().unwrap();
            if hold.holding {
                hold.held_exports += 1;
                changed.notify_all();
                drop(changed.wait_while(hold, |hold| hold.holding).unwrap());
            }
            if connection
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .is_err()
            {
                return;
            }
        }
    }

    fn hold_answers(&self) {
        self.hold.0.lock().unwrap().holding = true;
    }

    fn release_answers(&self) {
        self.hold.0.lock().unwrap().holding = false;
        self.hold.1.notify_all();
    }

    fn wait_until_an_export_is_held(&self) {
        let (hold, changed) = &*self.hold;
        let (hold, timeout) = changed
            .wait_timeout_while(hold.lock().unwrap(), Duration::from_secs(5), |hold| {
                hold.held_exports == 0
            })
            .unwrap();
        assert!(
            !timeout.timed_out(),
            "no export arrived while the collector was holding"
        );
        drop(hold);
    }

    /// Whether any export to `path` carried `text`. Protobuf stores strings as plain UTF-8.
    fn received(&self, path: &str, text: &str) -> bool {
        self.exports.lock().unwrap().iter().any(|export| {
            export.path == path && export.body.windows(text.len()).any(|window| window == text.as_bytes())
        })
    }

    /// The trace ID on the exported log record whose body is `message`, if that record carries one.
    fn trace_id_of_log(&self, message: &str) -> Option<Vec<u8>> {
        self.exported_records("/v1/logs").into_iter().find_map(|log_record| {
            let body = first_field(&log_record, LOG_RECORD_BODY)?;
            let trace_id = first_field(&log_record, LOG_RECORD_TRACE_ID)?;
            (first_field(body, ANY_VALUE_STRING)? == message.as_bytes()).then(|| trace_id.to_vec())
        })
    }

    /// The trace ID of the exported span named `name`.
    fn trace_id_of_span(&self, name: &str) -> Option<Vec<u8>> {
        self.exported_records("/v1/traces").into_iter().find_map(|span| {
            let trace_id = first_field(&span, SPAN_TRACE_ID)?;
            (first_field(&span, SPAN_NAME)? == name.as_bytes()).then(|| trace_id.to_vec())
        })
    }

    /// Every span or log record exported to `path`, still encoded.
    fn exported_records(&self, path: &str) -> Vec<Vec<u8>> {
        let exports = self.exports.lock().unwrap();
        exports
            .iter()
            .filter(|export| export.path == path)
            .flat_map(|export| nested_messages(&export.body, &PATH_TO_RECORDS))
            .map(<[u8]>::to_vec)
            .collect()
    }
}

// Field numbers from the OTLP protobuf definitions. An export request holds its spans or log records
// at the same place: resource group 1, scope group 2, record 2.
const PATH_TO_RECORDS: [u64; 3] = [1, 2, 2];
const SPAN_TRACE_ID: u64 = 1;
const SPAN_NAME: u64 = 5;
const LOG_RECORD_BODY: u64 = 5;
const LOG_RECORD_TRACE_ID: u64 = 9;
const ANY_VALUE_STRING: u64 = 1;

/// Every message found by following `path`, a chain of field numbers, down from `message`.
fn nested_messages<'a>(message: &'a [u8], path: &[u64]) -> Vec<&'a [u8]> {
    let Some((field, rest)) = path.split_first() else {
        return vec![message];
    };
    length_delimited_fields(message)
        .into_iter()
        .filter(|(number, _)| number == field)
        .flat_map(|(_, payload)| nested_messages(payload, rest))
        .collect()
}

fn first_field(message: &[u8], field: u64) -> Option<&[u8]> {
    length_delimited_fields(message)
        .into_iter()
        .find_map(|(number, payload)| (number == field).then_some(payload))
}

/// The strings, bytes and nested messages of a protobuf message, each with its field number.
/// Numeric fields are skipped.
fn length_delimited_fields(mut message: &[u8]) -> Vec<(u64, &[u8])> {
    let mut fields = Vec::new();
    while !message.is_empty() {
        let key = read_varint(&mut message);
        match key & 0b111 {
            0 => {
                read_varint(&mut message);
            }
            1 => message = &message[8..],
            2 => {
                let length = usize::try_from(read_varint(&mut message)).unwrap();
                let (payload, rest) = message.split_at(length);
                fields.push((key >> 3, payload));
                message = rest;
            }
            5 => message = &message[4..],
            wire_type => panic!("unexpected protobuf wire type {wire_type}"),
        }
    }
    fields
}

fn read_varint(message: &mut &[u8]) -> u64 {
    let mut value = 0;
    for shift in (0..).step_by(7) {
        let byte = message[0];
        *message = &message[1..];
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            break;
        }
    }
    value
}

fn read_export(reader: &mut BufReader<TcpStream>) -> Option<Export> {
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).ok()? == 0 {
        return None;
    }
    let path = request_line.split_whitespace().nth(1)?.to_owned();

    let mut content_length = 0;
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).ok()?;
        let header = header.trim();
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().ok()?;
        }
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).ok()?;
    Some(Export { path, body })
}

/// The server process. It is killed if a test fails before the server has exited.
struct Server {
    process: Child,
    port: u16,
}

impl Server {
    /// Starts the server on the database named by `DATABASE_URL`.
    fn start(collector: &FakeCollector) -> Self {
        Self::start_on(collector, &test_database_url())
    }

    fn start_on(collector: &FakeCollector, database_url: &str) -> Self {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let process = Command::new(env!("CARGO_BIN_EXE_gold-ballot"))
            .env("PORT", port.to_string())
            .env("DATABASE_URL", database_url)
            .env("OTEL_EXPORTER_OTLP_ENDPOINT", &collector.endpoint)
            .env("TELEMETRY_FLUSH_EACH_REQUEST", "true")
            .env_remove("SPA_DIR")
            .spawn()
            .unwrap();
        let server = Self { process, port };

        let deadline = Instant::now() + Duration::from_secs(10);
        while server
            .get("/api/health")
            .is_none_or(|response| !response.starts_with("HTTP/1.1 200"))
        {
            assert!(Instant::now() < deadline, "the server did not start");
            thread::sleep(Duration::from_millis(50));
        }
        server
    }

    /// Sends a request and leaves the connection open, so the response can be read later.
    fn send_get(&self, path: &str) -> Option<TcpStream> {
        let mut connection = TcpStream::connect(("127.0.0.1", self.port)).ok()?;
        write!(
            connection,
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )
        .ok()?;
        Some(connection)
    }

    fn get(&self, path: &str) -> Option<String> {
        read_response(self.send_get(path)?)
    }

    fn send_sigterm(&self) {
        let status = Command::new("kill")
            .args(["-TERM", &self.process.id().to_string()])
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn is_running(&mut self) -> bool {
        self.process.try_wait().unwrap().is_none()
    }

    fn wait_for_exit(&mut self, limit: Duration) -> ExitStatus {
        let deadline = Instant::now() + limit;
        loop {
            if let Some(status) = self.process.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < deadline, "the server did not exit within {limit:?}");
            thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn read_response(mut connection: TcpStream) -> Option<String> {
    let mut response = String::new();
    connection.read_to_string(&mut response).ok()?;
    Some(response)
}

fn test_database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL names the test Postgres")
}

/// A database of its own on the test Postgres, which a test can take away from a running server.
struct ScratchDatabase {
    name: &'static str,
}

impl ScratchDatabase {
    fn create(name: &'static str) -> Self {
        let database = Self { name };
        database.remove();
        run_on_test_postgres(format!("CREATE DATABASE {name}"));
        database
    }

    fn url(&self) -> String {
        let options: PgConnectOptions = test_database_url().parse().unwrap();
        options.database(self.name).to_url_lossy().into()
    }

    /// Drops the database and closes every connection to it.
    fn remove(&self) {
        run_on_test_postgres(format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", self.name));
    }
}

impl Drop for ScratchDatabase {
    fn drop(&mut self) {
        self.remove();
    }
}

/// Runs a statement this file wrote itself; nothing from outside reaches the SQL.
fn run_on_test_postgres(statement: String) {
    Runtime::new().unwrap().block_on(async {
        let mut connection = PgConnection::connect(&test_database_url()).await.unwrap();
        sqlx::raw_sql(AssertSqlSafe(statement))
            .execute(&mut connection)
            .await
            .unwrap();
    });
}

/// How many migrations the database at `database_url` records as applied.
fn applied_migration_count(database_url: &str) -> usize {
    let count: i64 = Runtime::new().unwrap().block_on(async {
        let mut connection = PgConnection::connect(database_url).await.unwrap();
        sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
            .fetch_one(&mut connection)
            .await
            .unwrap()
    });
    count.try_into().unwrap()
}

fn migration_file_count() -> usize {
    std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"))
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|extension| extension == "sql")
        })
        .count()
}

/// Needs the Postgres named by `DATABASE_URL`, like the other tests.
#[test]
fn rule_2_4_6_a_start_applies_every_pending_migration_before_it_serves() {
    let collector = FakeCollector::start();
    let empty_database = ScratchDatabase::create("rule_2_4_6_migrate_at_start");

    // `start_on` returns with the server's first answer.
    let _server = Server::start_on(&collector, &empty_database.url());

    assert_eq!(applied_migration_count(&empty_database.url()), migration_file_count());
    assert!(migration_file_count() > 0);
}

/// Needs the Postgres named by `DATABASE_URL`, like the other tests.
#[test]
fn rule_2_4_4_sigterm_lets_the_request_in_flight_finish_and_exports_its_span() {
    let collector = FakeCollector::start();
    let mut server = Server::start(&collector);

    // The server exports before each response, so a held export keeps the request in flight.
    collector.hold_answers();
    let in_flight = server.send_get("/api/health/ready").unwrap();
    collector.wait_until_an_export_is_held();

    server.send_sigterm();
    thread::sleep(Duration::from_millis(300));
    assert!(server.is_running(), "the server exited while a request was in flight");

    collector.release_answers();
    let response = read_response(in_flight).unwrap();
    assert!(
        response.starts_with("HTTP/1.1 204"),
        "the request in flight got: {response}"
    );

    // Cloud Run kills the container 10 seconds after SIGTERM.
    assert!(server.wait_for_exit(Duration::from_secs(10)).success());

    assert!(collector.received("/v1/traces", "GET /api/health/ready"));
    assert!(collector.received("/v1/traces", "postgres.ping"));
    // Logged after the last request finished, so only the flush at shutdown can have exported it.
    assert!(collector.received("/v1/logs", "every in-flight request has finished"));
}

/// Needs the Postgres named by `DATABASE_URL`, like the other tests.
#[test]
fn rule_2_7_3_a_log_event_carries_the_trace_id_of_its_request() {
    let collector = FakeCollector::start();
    let database = ScratchDatabase::create("rule_2_7_3_log_trace_id");
    let mut server = Server::start_on(&collector, &database.url());

    // Readiness fails once the database is gone, and the server logs why inside that request.
    database.remove();
    let response = server.get("/api/health/ready").unwrap();
    assert!(response.starts_with("HTTP/1.1 503"), "readiness answered: {response}");
    server.send_sigterm();
    assert!(server.wait_for_exit(Duration::from_secs(10)).success());

    let log_trace_id = collector
        .trace_id_of_log("the database did not answer")
        .expect("a log record with that message and a trace ID was exported");
    let request_trace_id = collector
        .trace_id_of_span("GET /api/health/ready")
        .expect("the request's span was exported");
    assert_eq!(log_trace_id.len(), 16, "a trace ID is 16 bytes");
    assert_eq!(log_trace_id, request_trace_id);
}
