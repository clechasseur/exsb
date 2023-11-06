//! Helpers for [`tracing`] support.

pub fn log_level_to_tracing_level(log_level: log::Level) -> tracing::Level {
    match log_level {
        log::Level::Error => tracing::Level::ERROR,
        log::Level::Warn => tracing::Level::WARN,
        log::Level::Info => tracing::Level::INFO,
        log::Level::Debug => tracing::Level::DEBUG,
        log::Level::Trace => tracing::Level::TRACE,
    }
}
