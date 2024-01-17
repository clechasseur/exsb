//! Error types for our program.
//!
//! Currently, the [`Error`] and [`Result`] types used by our program are mapped to those
//! exposed by [`anyhow`].

/// Error type used by the [`exsb`](crate) program.
///
/// Currently mapped to [`anyhow::Error`].
pub type Error = anyhow::Error;

/// Result type used by the [`exsb`](crate) program.
///
/// Currently mapped to [`anyhow::Result`] in order to use our [`Error`] type.
pub type Result<T> = anyhow::Result<T>;
