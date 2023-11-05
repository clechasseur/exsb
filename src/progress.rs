//! Helpers dealing with CLI progress bars.

use log::{log_enabled, Level};

#[derive(Debug, Clone)]
pub struct ProgressBar {
    bar: Option<indicatif::ProgressBar>,
}

impl ProgressBar {
    pub fn for_log_level(level: Level, len: usize) -> Self {
        Self { bar: log_enabled!(level).then(|| indicatif::ProgressBar::new(len as u64)) }
    }

    pub fn println<M>(&self, msg: M)
    where
        M: AsRef<str>,
    {
        self.run_if_some(|bar| bar.println(msg));
    }

    pub fn inc(&self, delta: usize) {
        self.run_if_some(|bar| bar.inc(delta as u64));
    }

    fn run_if_some<F>(&self, f: F)
    where
        F: FnOnce(&indicatif::ProgressBar),
    {
        if let Some(bar) = &self.bar {
            f(bar);
        }
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        self.run_if_some(|bar| bar.finish_and_clear());
    }
}
