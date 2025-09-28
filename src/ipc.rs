//! Inter-Process Communication (IPC) logic.
//!
//! This module defines the communication protocol between the `rinkle` CLI
//! client and the background daemon, using a structured, JSON-based protocol
//! over local sockets.

use std::{
	io::{
		self,
		BufRead,
		BufReader,
		Write,
	},
	path::Path,
};

use anyhow::{
	Context,
	Result,
};
use interprocess::{
	TryClone,
	local_socket::{
		GenericFilePath,
		ListenerNonblockingMode,
		ListenerOptions,
		ToFsName,
		prelude::*,
	},
};
use serde::{
	Deserialize,
	Serialize,
};
use tracing::{
	debug,
	info,
	warn,
};

// --- Protocol Definition ---

/// A request sent from a client to the daemon.
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
	/// Check if the service is alive.
	Ping,
	/// Request the daemon to shut down gracefully.
	Stop,
}

/// A response sent from the daemon back to a client.
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
	/// Acknowledgment for `Ping`.
	Pong,
	/// Generic acknowledgment of success.
	Ok,
	/// An error occurred while processing the request.
	Error(String),
}

// --- Client-side Logic ---

/// Sends a single request to the daemon and waits for a response.
pub fn send_request(socket_path: &Path, request: Request) -> Result<Response> {
	let name = socket_path.to_fs_name::<GenericFilePath>()?;
	let mut conn = LocalSocketStream::connect(name)
		.context("Failed to connect to daemon socket")?;

	// Serialize and send the request.
	let request_json = serde_json::to_string(&request)?;
	conn.write_all(request_json.as_bytes())?;
	// Add a newline to signal end of message.
	conn.write_all(b"\n")?;
	conn.flush()?;
	debug!(?request, "Sent request to daemon");

	// Read the response.
	let mut reader = BufReader::new(conn);
	let mut response_str = String::new();
	reader.read_line(&mut response_str)?;

	let response: Response =
		serde_json::from_str(&response_str).with_context(|| {
			format!("Failed to deserialize daemon response: '{response_str}'")
		})?;

	debug!(?response, "Received response from daemon");
	Ok(response)
}

// --- Server-side Logic ---

/// Listens for incoming IPC connections and handles them in a loop.
///
/// This function takes a handler closure that processes each valid request
/// and returns a corresponding response.
pub fn listen<F>(
	socket_path: &Path,
	mut handler: F,
	shutdown_signal: &std::sync::atomic::AtomicBool,
) -> Result<()>
where
	F: FnMut(Request) -> Response,
{
	let name = socket_path.to_fs_name::<GenericFilePath>()?;
	let listener = ListenerOptions::new().name(name).create_sync()?;

	info!(path = %socket_path.display(), "IPC server listening.");

	// Set a read timeout so the accept loop isn't blocked indefinitely,
	// allowing it to periodically check the shutdown signal.
	listener.set_nonblocking(ListenerNonblockingMode::Stream)?;

	loop {
		// Check for shutdown signal.
		if shutdown_signal.load(std::sync::atomic::Ordering::SeqCst) {
			info!("Shutdown signal received, stopping IPC listener.");
			break;
		}

		match listener.accept() {
			Ok(mut conn) => {
				debug!("Accepted new IPC connection.");
				// Handle the connection in a blocking manner.
				if let Err(e) = handle_connection(&mut conn, &mut handler) {
					warn!(error = %e, "Error handling IPC connection");
				}
			}
			Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
				// No incoming connection, wait a bit before checking again.
				std::thread::sleep(std::time::Duration::from_millis(100));
				continue;
			}
			Err(e) => {
				return Err(e.into());
			}
		}
	}

	Ok(())
}

/// Handles a single, accepted IPC connection.
fn handle_connection<F>(
	conn: &mut LocalSocketStream,
	handler: &mut F,
) -> Result<()>
where
	F: FnMut(Request) -> Response,
{
	let mut reader = BufReader::new(conn.try_clone()?);
	let writer = conn;

	let mut line = String::new();
	reader.read_line(&mut line)?;

	let response = match serde_json::from_str::<Request>(&line) {
		Ok(request) => {
			debug!(?request, "Handling IPC request");
			handler(request)
		}
		Err(e) => {
			warn!(error = %e, "Failed to deserialize IPC request");
			Response::Error("Invalid request format".to_string())
		}
	};

	let response_json = serde_json::to_string(&response)?;
	writer.write_all(response_json.as_bytes())?;
	writer.write_all(b"\n")?;
	writer.flush()?;

	Ok(())
}
