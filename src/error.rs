//! Error types for the crate

use anyhow;
use thiserror::Error;

/// The error type for this crate
#[derive(Error, Debug)]
pub enum Error {
	/// A generic error which should not appear in production code
	#[error(transparent)]
	Generic(#[from] anyhow::Error),
}

/// Convenience type alias for this crate's error type
pub type Result<T> = std::result::Result<T, Error>;
