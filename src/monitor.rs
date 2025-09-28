use std::{
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		atomic::{
			AtomicBool,
			Ordering,
		},
		mpsc::{
			self,
			Receiver,
		},
	},
	thread,
	time::Duration,
};

use anyhow::Result;
use notify::{
	EventKind,
	RecommendedWatcher,
	RecursiveMode,
	Watcher,
};
use tracing::{
	debug,
	error,
	info,
	warn,
};

use crate::{
	daemon,
	ipc,
	utils::socket_path,
};

// --- Constants ---
const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(100);

// --- Public API ---

/// Starts the monitor as a background daemon process.
pub fn start() -> Result<()> {
	info!("Requesting to start monitor daemon...");

	if daemon::get_running_pid()?.is_some() {
		info!("Monitor daemon is already running.");
		return Ok(());
	}

	daemon::start()?;

	// Wait a moment for the daemon to start up.
	thread::sleep(Duration::from_millis(200));
	if status() {
		info!("Daemon started successfully and is responsive.");
	} else {
		warn!(
			"Daemon process may have started, but it is not responsive. Check \
			 logs for details."
		);
	}

	Ok(())
}

/// Stops the running monitor daemon.
pub fn stop() -> Result<()> {
	info!("Requesting to stop monitor daemon...");
	let pid = if let Some(pid) = daemon::get_running_pid()? {
		pid
	} else {
		info!("Monitor is not running.");
		return Ok(());
	};

	// First, try a graceful shutdown via IPC.
	info!("Attempting graceful shutdown via IPC...");
	match ipc::send_request(&socket_path(), ipc::Request::Stop) {
		Ok(ipc::Response::Ok) => {
			info!("Daemon acknowledged stop request.");
		}
		Err(e) => {
			warn!(error = %e, "Failed to send graceful stop request via IPC. Will fall back to signal.");
		}
		_ => {
			warn!(
				"Received unexpected response to stop request. Will fall back \
				 to signal."
			);
		}
	}

	// Always ensure the process is stopped using the platform-specific method.
	daemon::stop(pid)?;

	let start_time = std::time::Instant::now();
	while start_time.elapsed() < STOP_TIMEOUT {
		if daemon::get_running_pid()?.is_none() {
			info!("Monitor process has terminated.");
			return Ok(());
		}
		thread::sleep(STOP_POLL_INTERVAL);
	}

	warn!(
		"Monitor process did not terminate after {}s.",
		STOP_TIMEOUT.as_secs()
	);
	Ok(())
}

/// Returns true if the monitor daemon is running and responsive.
pub fn status() -> bool {
	if !daemon::status() {
		return false;
	}
	// Process is running, now check for service responsiveness via IPC ping.
	if matches!(
		ipc::send_request(&socket_path(), ipc::Request::Ping),
		Ok(ipc::Response::Pong)
	) {
		true
	} else {
		warn!(
			"Monitor process is running but the service is unresponsive to \
			 ping."
		);
		false
	}
}

/// Runs the core service loop. This is called inside the daemon/detached
/// process.
pub fn run_service_loop() -> Result<()> {
	info!("Starting daemon service loop...");
	let sock_path = socket_path();

	if sock_path.exists() {
		let _ = std::fs::remove_file(&sock_path);
	}

	let shutdown_signal = Arc::new(AtomicBool::new(false));
	let (watcher_shutdown_tx, _watcher_shutdown_rx) = mpsc::channel::<()>();

	// Graceful shutdown for Unix via SIGTERM.
	#[cfg(unix)]
	{
		let mut signals = signal_hook::iterator::Signals::new([
			signal_hook::consts::SIGTERM,
		])?;
		let signal_clone = shutdown_signal.clone();
		thread::spawn(move || {
			if signals.forever().next().is_some() {
				info!("Received SIGTERM, initiating graceful shutdown.");
				signal_clone.store(true, Ordering::SeqCst);
			}
		});
	}

	let watcher_handle: Option<thread::JoinHandle<()>> = None;
	// if let Some(src_dir) = resolve_source_dir() {
	//     let thread_config_path = config_path.to_path_buf();
	//     watcher_handle = Some(spawn_watcher_thread(src_dir,
	// watcher_shutdown_rx, thread_config_path)); } else {
	//     warn!("No 'global.source_dir' configured – file watcher is
	// disabled."); }

	// The IPC handler closure.
	let shutdown_signal_clone = shutdown_signal.clone();
	let ipc_handler = move |request: ipc::Request| -> ipc::Response {
		match request {
			ipc::Request::Ping => ipc::Response::Pong,
			ipc::Request::Stop => {
				info!("Received Stop request via IPC.");
				shutdown_signal_clone.store(true, Ordering::SeqCst);
				ipc::Response::Ok
			}
		}
	};

	// Start the IPC listener. This will block until a shutdown is signaled.
	if let Err(e) = ipc::listen(&sock_path, ipc_handler, &shutdown_signal) {
		error!(error = %e, "IPC listener failed");
	}

	// Shut down the watcher thread and clean up.
	let _ = watcher_shutdown_tx.send(());
	if let Some(handle) = watcher_handle {
		info!("Waiting for watcher thread to exit...");
		handle.join().expect("Watcher thread panicked");
	}

	let _ = std::fs::remove_file(&sock_path);
	info!("Daemon service loop stopped cleanly.");
	Ok(())
}

fn _spawn_watcher_thread(
	path: PathBuf,
	shutdown_rx: Receiver<()>,
	_config_path: PathBuf,
) -> thread::JoinHandle<()> {
	info!(watch_path = %path.display(), "Spawning filesystem watcher thread.");
	thread::spawn(move || {
		if !path.exists() {
			warn!("Watch path does not exist: {}", path.display());
		}
		let (event_tx, event_rx) = mpsc::channel();

		let mut watcher: RecommendedWatcher = match RecommendedWatcher::new(
			move |res: notify::Result<notify::Event>| {
				if let Ok(ev) = res {
					let _ = event_tx.send(ev);
				}
			},
			notify::Config::default(),
		) {
			Ok(w) => w,
			Err(e) => {
				error!(error = %e, "Failed to create file watcher");
				return;
			}
		};

		if let Err(e) =
			watcher.watch(Path::new(&path), RecursiveMode::Recursive)
		{
			error!(error = %e, "Failed to start watching path");
			return;
		}

		let debounce_duration = Duration::from_millis(500);
		let mut needs_relink = false;

		loop {
			if shutdown_rx.try_recv().is_ok() {
				break;
			}
			match event_rx.recv_timeout(debounce_duration) {
				Ok(event) => {
					if matches!(
						event.kind,
						EventKind::Modify(_) |
							EventKind::Create(_) | EventKind::Remove(_)
					) {
						debug!(?event, "FS event received, scheduling relink.");
						needs_relink = true;
					}
				}
				Err(mpsc::RecvTimeoutError::Timeout) => {
					if needs_relink {
						// TODO: Implement the relink action.
						debug!(
							"Debounce timeout reached. File system event was \
							 detected, but relink action is currently \
							 disabled."
						);
						needs_relink = false;
					}
				}
				Err(mpsc::RecvTimeoutError::Disconnected) => {
					warn!(
						"Event channel disconnected, watcher thread exiting."
					);
					break;
				}
			}
		}
		info!("Watcher thread exiting.");
	})
}

/// A submodule to centralize path management for the daemon.
pub mod paths {}
