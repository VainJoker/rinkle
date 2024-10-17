use crate::{
	cli::Cli,
	prelude::*,
	utils,
};

#[derive(Debug, Clone)]
pub struct Rinkle {
	pub config: &'static Config,
}

impl Default for Rinkle {
	fn default() -> Self {
		Self::new()
	}
}

impl Rinkle {
	pub fn new() -> Self {
		let config = get_config();
		Self { config }
	}

	pub async fn run(self, _cli: &Cli) -> Result<()> {
		let content =
			utils::walk_dir(self.config.global.source_dir.as_ref()).await?;
		println!("{content:?}");
		Ok(())
	}
}
