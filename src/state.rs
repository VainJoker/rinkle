use std::{
	collections::HashMap,
	fs::{
		File,
		OpenOptions,
	},
	io::{
		Seek,
		SeekFrom,
		Write,
	},
	path::{
		Path,
		PathBuf,
	},
};

use fd_lock::RwLock;
use serde::{
	Deserialize,
	Serialize,
};
use thiserror::Error;

/// Errors that can occur during state loading or saving.
#[derive(Debug, Error)]
pub enum StateError {
	/// The user's configuration directory could not be determined.
	#[error("could not find user's config directory")]
	NoConfigDir,
	/// An I/O error occurred while reading or writing the state file.
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	/// The state file content could not be parsed as valid TOML.
	#[error("toml parse error: {0}")]
	TomlDe(#[from] toml::de::Error),
	/// The state struct could not be serialized into TOML.
	#[error("toml serialize error: {0}")]
	TomlSer(#[from] toml::ser::Error),
}

/// Represents the persistent state of the application.
///
/// This struct is serialized to `state.toml` to remember user choices across
/// sessions, such as the active profile and pinned package versions.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct State {
	/// The name of the currently active profile.
	pub active_profile:  Option<String>,
	/// A map of package names to their pinned versions.
	#[serde(default)]
	pub pinned_versions: HashMap<String, String>, // package_name -> version
}

/// Returns the default path for the `state.toml` file.
///
/// This is typically `~/.config/rinkle/state.toml`. It creates the directory
/// if it doesn't exist.
pub fn default_state_path() -> PathBuf {
	let mut path = dirs::config_dir().expect("failed to find config dir");
	path.push("rinkle");
	std::fs::create_dir_all(&path).expect("failed to create config dir");
	path.push("state.toml");
	path
}

fn open_state_file(path: &Path) -> Result<File, StateError> {
	OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(path)
		.map_err(StateError::Io)
}

/// Loads the application state from the state file.
///
/// This function acquires a read lock on the file, reads its content, and
/// deserializes it into a `State` struct. If the file is empty or does not
/// exist, it returns a default `State`.
pub fn load_state(path: &Path) -> Result<State, StateError> {
	let file = open_state_file(path)?;
	let lock = RwLock::new(file);
	let handle = lock.read()?; // read guard derefs to &File

	let mut content = String::new();
	{
		use std::io::Read;
		// SAFETY: We only need shared read, File::read requires &mut self, so clone the file via try_clone.
		let mut f = handle.try_clone()?;
		f.read_to_string(&mut content)?;
	}

	if content.is_empty() {
		return Ok(State::default());
	}

	let state: State = toml::from_str(&content)?;
	Ok(state)
}

/// Saves the application state to the state file.
///
/// This function acquires a write lock on the file, serializes the `State`
/// struct into a TOML string, and writes it to the file, overwriting any
// previous content.
pub fn save_state(
	path: &Path,
	state: &State,
) -> Result<(), StateError> {
	let file = open_state_file(path)?;
	let mut lock = RwLock::new(file);
	let mut handle = lock.write()?;

	let content = toml::to_string_pretty(state)?;
	handle.set_len(0)?;
	handle.seek(SeekFrom::Start(0))?;
	handle.write_all(content.as_bytes())?;
	Ok(())
}
