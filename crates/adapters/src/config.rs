//! Server configuration, read only from environment variables (spec rule 2.4.3).

use std::fmt;
use std::path::PathBuf;

const PORT: &str = "PORT";
const SPA_DIR: &str = "SPA_DIR";
const DATABASE_URL: &str = "DATABASE_URL";

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

/// What the HTTP server needs to start.
#[derive(Debug, PartialEq, Eq)]
pub struct ServerConfig {
    /// TCP port to listen on. Cloud Run supplies it as `PORT`.
    pub port: u16,
    /// Directory holding the built SPA. Unset in development, where the Vite dev server serves the frontend.
    pub spa_dir: Option<PathBuf>,
    /// Where Postgres is. In production it comes from the platform's secret store.
    pub database_url: DatabaseUrl,
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
        Ok(Self {
            port,
            spa_dir,
            database_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCAL_DATABASE: &str = "postgres://user:secret@localhost/gold_ballot";

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
            })
        );
    }

    #[test]
    fn rule_6_5_6_debug_output_hides_the_database_password() {
        let config = ServerConfig::from_lookup(environment(&[("PORT", "8080"), ("DATABASE_URL", LOCAL_DATABASE)]));

        assert!(!format!("{config:?}").contains("secret"));
    }
}
