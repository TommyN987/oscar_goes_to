//! Server configuration, read only from environment variables (spec rule 2.4.3).

use std::fmt;
use std::path::PathBuf;

const PORT: &str = "PORT";
const SPA_DIR: &str = "SPA_DIR";

/// What the HTTP server needs to start.
#[derive(Debug, PartialEq, Eq)]
pub struct ServerConfig {
    /// TCP port to listen on. Cloud Run supplies it as `PORT`.
    pub port: u16,
    /// Directory holding the built SPA. Unset in development, where the Vite dev server serves the frontend.
    pub spa_dir: Option<PathBuf>,
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
        Ok(Self { port, spa_dir })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_2_4_3_a_missing_port_is_named() {
        let error = ServerConfig::from_lookup(|_| None).unwrap_err();

        assert_eq!(error, ConfigError::Missing { variable: "PORT" });
        assert!(error.to_string().contains("PORT"));
    }

    #[test]
    fn rule_2_4_3_a_malformed_port_is_named() {
        let error = ServerConfig::from_lookup(|_| Some("eighty".into())).unwrap_err();

        assert!(matches!(error, ConfigError::Invalid { variable: "PORT", .. }));
    }

    #[test]
    fn the_spa_directory_is_optional() {
        let config = ServerConfig::from_lookup(|variable| (variable == "PORT").then(|| "8080".into()));

        assert_eq!(
            config,
            Ok(ServerConfig {
                port: 8080,
                spa_dir: None
            })
        );
    }
}
