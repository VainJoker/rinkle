use std::collections::HashMap;

use serde::{
	Deserialize,
	Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub global:   Global,
	pub log:      Log,
	pub ignore:   Ignore,
	pub ui:       UI,
	pub vsc:      Vsc,
	pub packages: HashMap<String, Package>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Global {
	pub source_dir:        String,
	pub target_dir:        String,
	pub link_strategy:     LinkStrategy,
	pub conflict_strategy: ConflictStrategy,
	pub monitor_interval:  u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
	pub log_level: LogLevel,
	pub log_file:  String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ignore {
	pub items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UI {
	pub use_color:        bool,
	pub progress_display: ProgressDisplay,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vsc {
	pub template: String,
	pub default:  String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
	#[serde(default)]
	pub source:      Option<String>,
	#[serde(default)]
	pub target:      Option<String>,
	#[serde(default)]
	pub vsc_default: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConflictStrategy {
	#[serde(rename = "skip")]
	Skip,
	#[serde(rename = "overwrite")]
	Overwrite,
	#[serde(rename = "backup")]
	Backup,
	#[serde(rename = "prompt")]
	Prompt,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LinkStrategy {
	#[serde(rename = "files")]
	Files,
	#[serde(rename = "directories")]
	Directories,
	#[serde(rename = "adaptive")]
	Adaptive,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LogLevel {
	#[serde(rename = "debug")]
	Debug,
	#[serde(rename = "info")]
	Info,
	#[serde(rename = "warn")]
	Warn,
	#[serde(rename = "error")]
	Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProgressDisplay {
	#[serde(rename = "bar")]
	Bar,
	#[serde(rename = "percentage")]
	Percentage,
	#[serde(rename = "none")]
	None,
}