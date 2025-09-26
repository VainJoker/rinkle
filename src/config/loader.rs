use std::path::{
	Path,
	PathBuf,
};

use thiserror::Error;

use crate::config::entity::Config;

/// Errors that can occur while loading the configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
	/// An I/O error occurred while reading the file.
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	/// The file content could not be parsed as valid TOML.
	#[error("toml parse error: {0}")]
	TomlDe(#[from] toml::de::Error),
}

/// Determines the default path for the `rinkle.toml` configuration file.
///
/// It prefers `./config/rinkle.toml`, but falls back to `./rinkle.toml` if the
/// first does not exist.
pub fn default_config_path() -> PathBuf {
	// Prefer ./config/rinkle.toml, fallback to ./rinkle.toml
	let path1 = PathBuf::from("config/rinkle.toml");
	if path1.exists() {
		return path1;
	}
	PathBuf::from("rinkle.toml")
}

/// Loads and parses a `rinkle.toml` file from the specified path.
///
/// # Arguments
///
/// * `path` - The path to the `rinkle.toml` file.
///
/// # Returns
///
/// A `Result` containing the parsed `Config` struct or a `ConfigError`.
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
	let content = std::fs::read_to_string(path)?;
	let cfg: Config = toml::from_str(&content)?;
	Ok(cfg)
}
