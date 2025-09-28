//! Windows-specific "daemon" implementation using a detached process.

use std::{
	fs,
	io::Write,
	os::windows::process::CommandExt,
	process::{
		Command,
		Stdio,
	},
	str::FromStr,
};

use anyhow::{
	Context,
	Result,
	anyhow,
};
use sysinfo::{
	Pid,
	System,
};
use tracing::{
	info,
	warn,
};

use crate::monitor::paths;

const DETACHED_PROCESS: u32 = 0x00000008;

/// Starts the detached process.
pub fn start(config_path: &Path) -> Result<()> {
	let current_exe = std::env::current_exe()
		.context("Failed to get current executable path")?;
	let pid_file = paths::pid_path();

	// Spawn the same executable with a hidden command to run the service loop.
	let mut cmd = Command::new(current_exe);
	cmd.arg("internal-run-service")
		.arg("--config")
		.arg(config_path)
		.stdin(Stdio::null())
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.creation_flags(DETACHED_PROCESS);

	let child = cmd.spawn().context("Failed to spawn detached process")?;

	// Write the PID to the file immediately.
	fs::File::create(&pid_file)?
		.write_all(child.id().to_string().as_bytes())
		.context("Failed to write PID file")?;

	info!("Detached process started with PID: {}", child.id());
	Ok(())
}

/// Stops the detached process.
pub fn stop(pid: Pid) -> Result<()> {
	info!("Stopping process with PID: {}", pid);
	let mut system = System::new_all();
	system.refresh_processes();

	if let Some(process) = system.process(pid) {
		if !process.kill() {
			return Err(anyhow!("Failed to kill process {}", pid));
		}
	} else {
		warn!(
			"Process with PID {} not found, it may have already exited.",
			pid
		);
	}
	Ok(())
}

/// Gets the PID of the running process, cleaning up stale files if it's dead.
pub fn get_running_pid() -> Result<Option<Pid>> {
	let pid_file = paths::pid_path();
	if !pid_file.exists() {
		return Ok(None);
	}

	let pid_str =
		fs::read_to_string(&pid_file).context("Failed to read PID file")?;
	let pid_val = u32::from_str(pid_str.trim())
		.map_err(|_| anyhow!("Invalid PID '{}' in file", pid_str))?;
	let pid = Pid::from(pid_val as usize);

	let mut system = System::new_all();
	system.refresh_processes();

	if system.process(pid).is_some() {
		Ok(Some(pid))
	} else {
		warn!(
			"Found stale PID file for dead process {}, cleaning up.",
			pid
		);
		let _ = fs::remove_file(&pid_file);
		let _ = fs::remove_file(paths::socket_path());
		Ok(None)
	}
}
