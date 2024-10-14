use std::{
	path::Path,
	sync::OnceLock,
};

use entity::Config;
use realme::{
	Adaptor,
	FileSource,
	Realme,
	TomlParser,
};

use crate::error::{
	Error,
	Result,
};

mod entity;

pub static CFG: OnceLock<Config> = OnceLock::new();

pub fn initialize_config(path: &'static Path) -> Result<&'static Config> {
	let config = CFG.get_or_init(|| {
		Config::load_config(path).unwrap_or_else(|e| {
			eprintln!("Load config err: {e}");
			tracing::error!("Load config err: {e}");
			std::process::exit(78);
		})
	});
	Ok(config)
}

pub fn get_config() -> &'static Config {
	CFG.get().expect("Config not initialized")
}

impl Config {
	pub fn load_config(path: &'static Path) -> Result<Self> {
		let realme = Realme::builder()
			.load(Adaptor::new(FileSource::<TomlParser, _>::new(path)))
			.build()
			.map_err(|e| Error::Config(e.to_string()))?;

		eprintln!("realme: {realme:#?}");
		let config = realme
			.try_deserialize()
			.map_err(|e| Error::Config(e.to_string()))?;
		Ok(config)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_config() {
		let res = initialize_config(Path::new("./config/rinkle.toml"));
		assert!(res.is_ok());
	}
}
