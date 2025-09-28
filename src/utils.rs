use std::path::PathBuf;

pub const fn current_os() -> &'static str {
	if cfg!(target_os = "linux") {
		"linux"
	} else if cfg!(target_os = "macos") {
		"macos"
	} else {
		"other"
	}
}

pub fn pid_path() -> PathBuf {
	std::env::temp_dir().join("rinkle-monitor.pid")
}
pub fn socket_path() -> PathBuf {
	std::env::temp_dir().join("rinkle-monitor.sock")
}
pub fn stdout_log_path() -> PathBuf {
	std::env::temp_dir().join("rinkle-daemon.out")
}
pub fn stderr_log_path() -> PathBuf {
	std::env::temp_dir().join("rinkle-daemon.err")
}
