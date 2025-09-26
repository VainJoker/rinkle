use std::{
	sync::mpsc::channel,
	time::{
		Duration,
		Instant,
	},
};

use notify::{
	EventKind,
	RecommendedWatcher,
	RecursiveMode,
	Watcher,
};
use tracing::{
	debug,
	info,
	warn,
};

use crate::{
	config::entity::Config,
	linker,
};

/// Runs the file system monitor in the foreground.
///
/// This function watches the `source_dir` for changes. In its current MVP
/// state, it only logs a message when a change is detected, after a simple
/// debounce delay. It does not automatically trigger any actions.
pub fn run_foreground(cfg: &Config) -> std::io::Result<()> {
	let source_root = cfg.global.source_dir.as_deref().unwrap_or(".");
	let path = linker::expand_path(source_root);
	let (tx, rx) = channel();
	let mut watcher: RecommendedWatcher =
		Watcher::new(tx, notify::Config::default()).map_err(to_io)?;
	watcher
		.watch(&path, RecursiveMode::Recursive)
		.map_err(to_io)?;
	info!("monitoring {}", path.display());

	// simple debounce
	let mut last = Instant::now();
	loop {
		match rx.recv() {
			Ok(Ok(evt)) => {
				debug!(?evt, "fs event");
				if !matches!(
					evt.kind,
					EventKind::Modify(_) |
						EventKind::Create(_) |
						EventKind::Remove(_)
				) {
					continue;
				}
				if last.elapsed() < Duration::from_millis(500) {
					continue;
				}
				last = Instant::now();
				// MVP: just print; 后续可触发 link 当前 profile 的包
				info!("change detected; run 'rinkle link' if desired");
			}
			Ok(Err(e)) => warn!("watch error: {e}"),
			Err(e) => warn!("channel error: {e}"),
		}
	}
}

/// Stops the file monitor.
///
/// (MVP) This is a placeholder and is not yet implemented.
pub fn stop() -> std::io::Result<()> {
	// MVP: 无 PID 管理，留空实现
	println!("stop requested (not implemented)");
	Ok(())
}

fn to_io<E: std::error::Error + Send + Sync + 'static>(e: E) -> std::io::Error {
	std::io::Error::new(std::io::ErrorKind::Other, e)
}
