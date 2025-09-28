//! Cross-platform daemon management.
//!
//! This module abstracts the platform-specific details of starting, stopping,
//! and checking the status of a background process (a "daemon" on Unix, or a
//! detached process on Windows).

// Conditionally compile and export the correct implementation.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::{
	get_running_pid,
	start,
	stop,
};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{
	get_running_pid,
	start,
	stop,
};

/// A platform-agnostic representation of a process ID.
#[cfg(windows)]
pub type Pid = sysinfo::Pid;

/// Checks if the daemon is running and the service is responsive.
pub fn status() -> bool {
	// This part is platform-agnostic: if we have a running PID, we check the
	// IPC socket.
	if get_running_pid().unwrap_or(None).is_none() {
		return false;
	}
	// The rest of the logic (pinging the socket) is handled in the monitor
	// module.
	true
}
