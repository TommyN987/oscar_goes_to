//! Server configuration, read only from environment variables (spec rule 2.4.3).

use std::fmt;
use std::path::PathBuf;

const PORT: &str = "PORT";
const SPA_DIR: &str = "SPA_DIR";
const DATABASE_URL: &str = "DATABASE_URL";
const OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
const TELEMETRY_FLUSH_EACH_REQUEST: &str = "TELEMETRY_FLUSH_EACH_REQUEST";

/// A Postgres connection string. It holds the password, so its `Debug` output hides the value.
#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseUrl(String);

impl DatabaseUrl {
    /// The connection string itself. Pass it only to the database driver; never log it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DatabaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DatabaseUrl(<redacted>)")
    }
}

/// How telemetry leaves the process.
#[derive(Debug, PartialEq, Eq)]
pub struct TelemetryConfig {
    /// Export over OTLP. On when `OTEL_EXPORTER_OTLP_ENDPOINT` is set; the exporter reads that variable itself.
    pub export: bool,
    /// Export before each response, not in the background. A switch for debugging on Cloud Run,
    /// where background work pauses between requests (spec rule 2.7.5).
    pub flush_each_request: bool,
}

/// What the HTTP server needs to start.
#[derive(Debug, PartialEq, Eq)]
pub struct ServerConfig {
    /// TCP port to listen on. Cloud Run supplies it as `PORT`.
    pub port: u16,
    /// Directory holding the built SPA. Unset in development, where the Vite dev server serves the frontend.
    pub spa_dir: Option<PathBuf>,
    /// Where Postgres is. In production it comes from the platform's secret store.
    pub database_url: DatabaseUrl,
    pub telemetry: TelemetryConfig,
}

/// A configuration variable that is missing or malformed. Its message names the variable.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    Missing { variable: &'static str },
    Invalid { variable: &'static str, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { variable } => write!(f, "environment variable {variable} is not set"),
            Self::Invalid { variable, reason } => {
                write!(f, "environment variable {variable} is invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl ServerConfig {
    /// Reads the configuration from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|variable| std::env::var(variable).ok())
    }

    /// Reads the configuration through `lookup`, which returns a variable's value if it is set.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let port = lookup(PORT)
            .ok_or(ConfigError::Missing { variable: PORT })?
            .parse()
            .map_err(|error: std::num::ParseIntError| ConfigError::Invalid {
                variable: PORT,
                reason: error.to_string(),
            })?;
        let spa_dir = lookup(SPA_DIR).map(PathBuf::from);
        let database_url = lookup(DATABASE_URL)
            .map(DatabaseUrl)
            .ok_or(ConfigError::Missing { variable: DATABASE_URL })?;
        let flush_each_request = match lookup(TELEMETRY_FLUSH_EACH_REQUEST).as_deref() {
            None | Some("false") => false,
            Some("true") => true,
            Some(other) => {
                return Err(ConfigError::Invalid {
                    variable: TELEMETRY_FLUSH_EACH_REQUEST,
                    reason: format!("expected `true` or `false`, found `{other}`"),
                });
            }
        };
        let telemetry = TelemetryConfig {
            export: lookup(OTLP_ENDPOINT).is_some_and(|endpoint| !endpoint.is_empty()),
            flush_each_request,
        };
        Ok(Self {
            port,
            spa_dir,
            database_url,
            telemetry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL_DATABASE: &str = "postgres://user:secret@localhost/gold_ballot";
    const ENV_EXAMPLE: &str = include_str!("../../../.env.example");
    const JUSTFILE: &str = include_str!("../../../justfile");

    /// Whether `.env.example` has a line for `variable`, set or commented out.
    fn env_example_names(variable: &str) -> bool {
        ENV_EXAMPLE.lines().any(|line| {
            line.trim_start_matches("# ")
                .split_once('=')
                .is_some_and(|(name, _)| name == variable)
        })
    }

    fn environment(variables: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |name| {
            variables
                .iter()
                .find(|(variable, _)| *variable == name)
                .map(|(_, value)| value.to_string())
        }
    }

    #[test]
    fn rule_2_4_3_a_missing_port_is_named() {
        let error = ServerConfig::from_lookup(environment(&[("DATABASE_URL", LOCAL_DATABASE)])).unwrap_err();

        assert_eq!(error, ConfigError::Missing { variable: "PORT" });
        assert!(error.to_string().contains("PORT"));
    }

    #[test]
    fn rule_2_4_3_a_malformed_port_is_named() {
        let error = ServerConfig::from_lookup(environment(&[("PORT", "eighty"), ("DATABASE_URL", LOCAL_DATABASE)]))
            .unwrap_err();

        assert!(matches!(error, ConfigError::Invalid { variable: "PORT", .. }));
    }

    #[test]
    fn rule_2_4_3_a_missing_database_url_is_named() {
        let error = ServerConfig::from_lookup(environment(&[("PORT", "8080")])).unwrap_err();

        assert_eq!(
            error,
            ConfigError::Missing {
                variable: "DATABASE_URL"
            }
        );
        assert!(error.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn the_spa_directory_is_optional() {
        let config = ServerConfig::from_lookup(environment(&[("PORT", "8080"), ("DATABASE_URL", LOCAL_DATABASE)]));

        assert_eq!(
            config,
            Ok(ServerConfig {
                port: 8080,
                spa_dir: None,
                database_url: DatabaseUrl(LOCAL_DATABASE.into()),
                telemetry: TelemetryConfig {
                    export: false,
                    flush_each_request: false,
                },
            })
        );
    }

    #[test]
    fn telemetry_is_exported_when_an_otlp_endpoint_is_set() {
        let config = ServerConfig::from_lookup(environment(&[
            ("PORT", "8080"),
            ("DATABASE_URL", LOCAL_DATABASE),
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318"),
            ("TELEMETRY_FLUSH_EACH_REQUEST", "true"),
        ]))
        .unwrap();

        assert_eq!(
            config.telemetry,
            TelemetryConfig {
                export: true,
                flush_each_request: true,
            }
        );
    }

    #[test]
    fn rule_2_4_3_a_malformed_flush_switch_is_named() {
        let error = ServerConfig::from_lookup(environment(&[
            ("PORT", "8080"),
            ("DATABASE_URL", LOCAL_DATABASE),
            ("TELEMETRY_FLUSH_EACH_REQUEST", "yes"),
        ]))
        .unwrap_err();

        assert!(error.to_string().contains("TELEMETRY_FLUSH_EACH_REQUEST"));
    }

    #[test]
    fn rule_2_1_4_env_example_names_every_variable_the_configuration_reads() {
        let read = std::cell::RefCell::new(Vec::new());
        // The required variables get valid values, so no early error hides a later read.
        ServerConfig::from_lookup(|variable| {
            read.borrow_mut().push(variable.to_owned());
            match variable {
                PORT => Some("8080".into()),
                DATABASE_URL => Some(LOCAL_DATABASE.into()),
                _ => None,
            }
        })
        .expect("give every newly required variable a valid value in this test");

        let unnamed: Vec<String> = read
            .into_inner()
            .into_iter()
            .filter(|variable| !env_example_names(variable))
            .collect();
        assert!(unnamed.is_empty(), ".env.example does not name {unnamed:?}");
    }

    #[test]
    fn rule_2_1_4_env_example_names_every_variable_the_deploy_sets() {
        let set_by_the_deploy = JUSTFILE
            .lines()
            .find_map(|line| line.trim().strip_prefix("--set-secrets="))
            .expect("the deploy recipe passes --set-secrets")
            .trim_end_matches('\\')
            .trim();

        let unnamed: Vec<&str> = set_by_the_deploy
            .split(',')
            .map(|assignment| assignment.split_once('=').expect("NAME=secret:version").0)
            .filter(|variable| !env_example_names(variable))
            .collect();
        assert!(unnamed.is_empty(), ".env.example does not name {unnamed:?}");
    }

    #[test]
    fn rule_6_5_6_debug_output_hides_the_database_password() {
        let config = ServerConfig::from_lookup(environment(&[("PORT", "8080"), ("DATABASE_URL", LOCAL_DATABASE)]));

        assert!(!format!("{config:?}").contains("secret"));
    }
}
