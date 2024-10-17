//! Prelude for the crate
pub use crate::{
	config::{
		Config,
		get_config,
	},
	error::*,
	rinkle::Rinkle,
};

#[allow(dead_code)]
/// Generic Wrapper tuple struct for newtype patterns
pub struct W<T>(pub T);
