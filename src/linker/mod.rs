// linker/mod.rs
// This module re-exports the actual linker implementation for use in the rest of the codebase.

mod linker_impl;

pub use linker_impl::*;
