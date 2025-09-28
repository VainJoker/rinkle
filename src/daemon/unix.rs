//! Unix-specific daemon implementation using `daemonize` and `nix`.

use std::{
	fs,
	str::FromStr,
};

use anyhow::{
	Context,
	Result,
	anyhow,
};
use daemonize::Daemonize;
use nix::{
	sys::signal::{
		self,
		Signal,
	},
	unistd::Pid,
};
use tracing::{
	info,
	warn,
};

use crate::{
	monitor::run_service_loop,
	utils::{
		pid_path,
		socket_path,
		stderr_log_path,
		stdout_log_path,
	},
};

/// Starts the daemon process.
pub fn start() -> Result<()> {
	let pid_file = pid_path();
	let stdout = fs::File::create(stdout_log_path())?;
	let stderr = fs::File::create(stderr_log_path())?;

	let daemonize = Daemonize::new()
		.pid_file(&pid_file)
		.chown_pid_file(true)
		.working_directory(std::env::temp_dir())
		.stdout(stdout)
		.stderr(stderr);

	match daemonize.start() {
		Ok(()) => {
			// This code runs in the detached daemon process.
			info!("Daemon process started successfully.");
			if let Err(e) = run_service_loop() {
				// Use a more specific error message for the daemon context.
				tracing::error!(error = %e, "Daemon service loop failed");
				std::process::exit(1);
			}
		}
		Err(e) => return Err(e.into()),
	}
	Ok(())
}

/// Stops the daemon process.
pub fn stop(pid: Pid) -> Result<()> {
	info!("Stopping daemon process with PID: {}", pid);
	signal::kill(pid, Signal::SIGTERM)?;
	Ok(())
}

/// Gets the PID of the running daemon, cleaning up stale files if it's dead.
pub fn get_running_pid() -> Result<Option<Pid>> {
	let pid_file = pid_path();
	if !pid_file.exists() {
		return Ok(None);
	}

	let pid_str =
		fs::read_to_string(&pid_file).context("Failed to read PID file")?;
	let pid_val = i32::from_str(pid_str.trim())
		.map_err(|_| anyhow!("Invalid PID '{pid_str}' in file"))?;
	let pid = Pid::from_raw(pid_val);

	// `kill -0` checks for process existence.
	if signal::kill(pid, None).is_ok() {
		Ok(Some(pid))
	} else {
		warn!(
			"Found stale PID file for dead process {}, cleaning up.",
			pid
		);
		let _ = fs::remove_file(&pid_file);
		let _ = fs::remove_file(socket_path());
		Ok(None)
	}
}
