//! The binary as a process, exporting to a fake OTLP collector: what SIGTERM does to a request in flight.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, ExitStatus};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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
    fn start(collector: &FakeCollector) -> Self {
        let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let process = Command::new(env!("CARGO_BIN_EXE_gold-ballot"))
            .env("PORT", port.to_string())
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
