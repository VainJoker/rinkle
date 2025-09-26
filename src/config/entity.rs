use std::collections::HashMap;

use serde::Deserialize;

/// Represents the `[global]` section of the `rinkle.toml` config.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Global {
	/// The root directory where the source dotfiles are located.
	pub source_dir:        Option<String>,
	/// The default directory where symlinks will be created.
	pub target_dir:        Option<String>,
	/// The default strategy to use when a symlink target already exists.
	#[serde(default = "default_conflict_strategy")]
	pub conflict_strategy: ConflictStrategy,
	/// A list of glob patterns to ignore when linking. (Not yet implemented)
	#[serde(default)]
	pub ignore:            Vec<String>,
}

fn default_conflict_strategy() -> ConflictStrategy {
	ConflictStrategy::Backup
}

/// Represents the `[vsc]` (Version Selection Control) section of the config.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Vsc {
	/// A regex used to identify versioned packages from directory names.
	/// It must contain a capture group named `version`.
	pub template:        Option<String>,
	/// The default version to use for packages if not otherwise specified.
	pub default_version: Option<String>,
}

/// Represents the `[profiles]` section of the config.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Profiles {
	/// The default profile, a list of tags to activate when no other profile
	/// is active.
	#[serde(default)]
	pub default: Vec<String>,
	// Additional named profiles are captured by serde's flatten attribute in
	// the `Config` struct.
}

/// Represents a single package defined under the `[packages]` section.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Package {
	/// Overrides the package's source path relative to `global.source_dir`.
	pub source:          Option<String>,
	/// Overrides the package's target path, making it an absolute path.
	pub target:          Option<String>,
	/// A list of operating systems this package should be applied on.
	/// (e.g., "linux", "macos")
	#[serde(default)]
	pub os:              Vec<String>,
	/// A list of tags used to group this package into profiles.
	#[serde(default)]
	pub tags:            Vec<String>,
	/// A package-specific default version.
	pub default_version: Option<String>,
}

/// The top-level structure representing the entire `rinkle.toml` configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
	/// Global configuration settings.
	#[serde(default)]
	pub global:   Global,
	/// Version Selection Control configuration.
	#[serde(default)]
	pub vsc:      Vsc,
	/// A map of profile names to lists of tags.
	#[serde(default)]
	pub profiles: HashMap<String, Vec<String>>, // profile name -> tags
	/// A map of package names to their configurations.
	#[serde(default)]
	pub packages: HashMap<String, Package>,
}

/// Defines the strategy for handling conflicts when a target file already
/// exists.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
	/// Do not create the symlink and print a warning.
	Skip,
	/// Remove the existing file/directory before creating the symlink.
	Overwrite,
	/// Rename the existing file/directory with a `.bak` suffix.
	Backup,
	/// Prompt the user for action. (Not yet implemented)
	Prompt,
}

impl Default for ConflictStrategy {
	fn default() -> Self {
		Self::Backup
	}
}
