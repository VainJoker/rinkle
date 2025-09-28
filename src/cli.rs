use std::path::PathBuf;

use clap::{
	Parser,
	Subcommand,
};

/// A symlink farm manager for your dotfiles.
///
/// Rinkle helps manage distinct sets of software and/or data located in
/// separate directories and makes them appear to be installed in a single
/// directory tree.
#[derive(Debug, Parser)]
#[command(name = "rk")]
#[command(version, about = "Symlink farm manager", long_about = None)]
pub struct Cli {
	/// Path to the rinkle.toml config file.
	///
	/// If not provided, rinkle searches for `config/rinkle.toml` and then
	/// `rinkle.toml` in the current directory.
	#[arg(long, global = true)]
	pub config: Option<PathBuf>,

	/// Perform a dry run.
	///
	/// This will simulate the command without making any actual changes to the
	/// filesystem, printing the actions that would have been taken.
	#[arg(long, global = true, default_value_t = false)]
	pub dry_run: bool,

	/// Override the active profile for a single command.
	///
	/// This is useful for temporarily linking or checking the status of a
	/// different profile without changing the saved state.
	#[arg(long, global = true)]
	pub profile: Option<String>,

	/// The subcommand to execute.
	#[command(subcommand)]
	pub command: Commands,
}

/// The set of available subcommands for Rinkle.
#[derive(Debug, Subcommand)]
pub enum Commands {
	/// List all packages defined in the configuration file.
	List,
	/// Show the link status of packages for the active profile.
	Status,
	/// Create symlinks for packages.
	///
	/// If no package names are provided, this command will link all packages
	/// belonging to the currently active profile.
	Link {
		/// The specific packages to link. Supports `package@version` syntax.
		packages: Vec<String>,
	},
	/// Remove symlinks for packages.
	///
	/// If no package names are provided, this command will remove links for all
	/// packages belonging to the currently active profile.
	Remove {
		/// The specific packages to remove.
		packages: Vec<String>,
	},
	/// Set the active profile.
	///
	/// The active profile determines which packages are processed by default
	/// for commands like `link`, `remove`, and `status`.
	UseProfile {
		/// The name of the profile to activate.
		name: String,
	},
	/// Pin a specific version for a version-controlled package.
	///
	/// This pins the version in the state file, so it will be used by default
	/// in subsequent commands.
	Vsc {
		/// The name of the package to pin.
		package: String,
		/// The version to pin (e.g., "stable", "nightly", "v1.2.3").
		version: String,
	},
	/// Initialize a new rinkle setup in the current directory.
	///
	/// This can clone a dotfiles repository and/or create a new `rinkle.toml`
	/// configuration file through an interactive prompt.
	Init {
		/// A git repository URL to clone into the destination directory.
		repo: Option<String>,
		/// The destination directory for the git clone. Defaults to the
		/// current directory.
		#[arg(long)]
		dest: Option<PathBuf>,
	},
	/// Enter interactive REPL mode.
	///
	/// This provides a shell for running rinkle commands.
	Interactive,
	/// Start the file monitor to watch for changes in the source directory.
	Start,
	/// Stop the background file monitor.
	Stop,
	/// [internal] Run the monitor service loop (used by the daemon).
	#[command(hide = true)]
	Run,
}
