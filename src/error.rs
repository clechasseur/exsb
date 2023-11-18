//! Error types for our program.
//!
//! Currently, the [`Error`] and [`Result`] types used by our program are mapped to those
//! exposed by [`anyhow`].

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;
