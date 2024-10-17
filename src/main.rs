use rinkle::{
	cli::Cli,
	config,
	prelude::*,
	rinkle::Rinkle,
};

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse_cli();
	let config_path = cli
		.config
		.as_ref()
		.map(std::path::PathBuf::from)
		.or_else(|| {
			dirs::config_local_dir()
				.map(|dir| dir.join("rinkle").join("rinkle.toml"))
		});

	if let Some(path) = config_path {
		config::initialize_config(
			&path,
			cli.config_args.as_deref().unwrap_or(""),
		)
		.expect("Failed to initialize config");
	}

	let rinkle = Rinkle::new();

	rinkle.run(&cli).await.expect("Failed to run rinkle");

	Ok(())
}
