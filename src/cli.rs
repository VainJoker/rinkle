//! CLI for Rinkle
//!
//! This module defines the command-line interface for the Rinkle tool.
//! It provides various subcommands and options for managing packages.

/// Execute all operations specified in config/rinkle.toml
///
/// Usage: `rk [package1] [package2] ...`
///
/// List all packages in config/rinkle.toml
///
/// Usage: `rk list`
///
/// Remove linked packages in `target_dir`
///
/// Usage: `rk remove [package1] [package2] ...`
///
/// Remove all linked packages in `target_dir`
///
/// Usage: `rk remove --all`
///
/// Clean all broken links in `target_dir`
///
/// Usage: `rk clean`
///
/// Check status of all packages in config/rinkle.toml
///
/// Usage: `rk status`
///
/// Select version for package
///
/// Usage: `rk vsc [package] [version]`
///
/// Start monitoring all packages in config/rinkle.toml
///
/// This will automatically link packages that have been changed after 5
/// seconds (configurable in config/rinkle.toml)
///
/// Usage: `rk start`
///
/// Stop monitoring all packages in config/rinkle.toml
///
/// Usage: `rk stop`
///
/// Enter interactive mode
///
/// Usage: `rk interactive`
///
/// Perform a dry run of all packages in config/rinkle.toml
///
/// Usage: `rk --dry-run`
///
/// Enable verbose output mode
///
/// Usage: `rk -vvv`
///
/// Use a custom configuration file
///
/// Usage: `rk --config ./config/rinkle.toml`
///
/// Display help information
///
/// Usage: `rk --help`
///
/// Display version information
///
/// Usage: `rk --version`
use clap::{
	Args,
	Parser,
	Subcommand,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Option<Commands>,

	/// Use a custom configuration file
	#[arg(short('c'), long, value_name = "FILE")]
	pub config: Option<String>,

	/// Specify config arguments
	#[arg(short('C'), long, value_name = "ARGS")]
	pub config_args: Option<String>,

	/// Enable verbose output mode
	#[arg(short, long, action = clap::ArgAction::Count)]
	pub verbose: u8,

	/// Perform a dry run without making any actual changes
	#[arg(long)]
	pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
	/// Link specified packages or all packages if none specified
	Link {
		/// List of package names to link
		packages: Vec<String>,
	},
	/// List all packages in the configuration file
	List,
	/// Remove linked packages
	Remove(RemoveArgs),
	/// Clean all broken links in the target directory
	Clean,
	/// Check status of all packages in the configuration
	Status,
	/// Select version for a specific package
	VSC {
		/// Package name
		package: String,
		/// Version number
		version: String,
	},
	/// Start monitoring all packages for changes
	Start,
	/// Stop monitoring packages for changes
	Stop,
	/// Enter interactive mode
	Interactive,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct RemoveArgs {
	/// List of package names to remove
	#[arg(group = "remove_option")]
	packages: Vec<String>,

	/// Remove all linked packages
	#[arg(long, group = "remove_option")]
	all: bool,
}

// Implement function to parse command line arguments
impl Cli {
	pub fn parse_cli() -> Self {
		Self::parse()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_link_command() {
		let cli = Cli::parse_from(["rk", "link", "package1", "package2"]);
		match cli.command {
			Some(Commands::Link { packages }) => {
				assert_eq!(packages, vec!["package1", "package2"]);
			}
			_ => panic!("Expected Link command"),
		}
	}

	#[test]
	fn test_list_command() {
		let cli = Cli::parse_from(["rk", "list"]);
		assert!(matches!(cli.command, Some(Commands::List)));
	}

	#[test]
	fn test_version_select_command() {
		let cli = Cli::parse_from(["rk", "vsc", "package1", "1.0.0"]);
		match cli.command {
			Some(Commands::VSC { package, version }) => {
				assert_eq!(package, "package1");
				assert_eq!(version, "1.0.0");
			}
			_ => panic!("Expected VersionSelect command"),
		}
	}

	#[test]
	fn test_global_flags() {
		let cli = Cli::parse_from([
			"rk",
			"-vvv",
			"--config",
			"custom.toml",
			"--dry-run",
			"list",
		]);
		assert_eq!(cli.verbose, 3);
		assert_eq!(cli.config, Some("custom.toml".to_string()));
		assert!(cli.dry_run);
		assert!(matches!(cli.command, Some(Commands::List)));
	}

	#[test]
	fn test_clean_command() {
		let cli = Cli::parse_from(["rk", "clean"]);
		assert!(matches!(cli.command, Some(Commands::Clean)));
	}

	#[test]
	fn test_status_command() {
		let cli = Cli::parse_from(["rk", "status"]);
		assert!(matches!(cli.command, Some(Commands::Status)));
	}

	#[test]
	fn test_start_command() {
		let cli = Cli::parse_from(["rk", "start"]);
		assert!(matches!(cli.command, Some(Commands::Start)));
	}

	#[test]
	fn test_stop_command() {
		let cli = Cli::parse_from(["rk", "stop"]);
		assert!(matches!(cli.command, Some(Commands::Stop)));
	}

	#[test]
	fn test_interactive_command() {
		let cli = Cli::parse_from(["rk", "interactive"]);
		assert!(matches!(cli.command, Some(Commands::Interactive)));
	}

	#[test]
	fn test_link_without_packages() {
		let cli = Cli::parse_from(["rk", "link"]);
		match cli.command {
			Some(Commands::Link { packages }) => {
				assert!(packages.is_empty());
			}
			_ => panic!("Expected Link command with empty packages"),
		}
	}

	#[test]
	fn test_remove_without_all_flag() {
		let cli = Cli::parse_from(["rk", "remove", "package1", "package2"]);
		match cli.command {
			Some(Commands::Remove(RemoveArgs { packages, all })) => {
				assert_eq!(packages, vec!["package1", "package2"]);
				assert!(!all);
			}
			_ => panic!("Expected Remove command without --all flag"),
		}
	}

	#[test]
	fn test_verbose_levels() {
		let cli = Cli::parse_from(["rk", "-v", "list"]);
		assert_eq!(cli.verbose, 1);

		let cli = Cli::parse_from(["rk", "-vv", "list"]);
		assert_eq!(cli.verbose, 2);

		let cli = Cli::parse_from(["rk", "-vvv", "list"]);
		assert_eq!(cli.verbose, 3);
	}
}
