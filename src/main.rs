mod app;
mod cli;
mod config;
mod error; // expose crate::error for internal modules
mod linker;
mod monitor;
mod repl;
mod setup;
mod state;

use clap::Parser;
use tracing::error;
use tracing_subscriber::{
	EnvFilter,
	fmt,
};

use crate::cli::Cli;

fn init_tracing() {
	let filter = EnvFilter::try_from_default_env()
		.unwrap_or_else(|_| EnvFilter::new("info"));
	fmt().with_env_filter(filter).init();
}

fn main() {
	init_tracing();
	let cli = Cli::parse();
	let app = app::App::new(&cli);

	if let Err(e) = app.run(&cli.command) {
		error!("command failed: {e}");
		std::process::exit(1);
	}
}
