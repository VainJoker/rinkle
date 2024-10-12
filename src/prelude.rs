//! Prelude for the crate
pub use crate::error::*;

#[allow(dead_code)]
/// Generic Wrapper tuple struct for newtype patterns
pub struct W<T>(pub T);
