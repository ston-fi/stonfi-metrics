use std::ops::Deref;
use std::sync::{Mutex, OnceLock};

/// Fallibly initialized storage for module-owned metrics.
///
/// Register this with [`crate::register_metrics!`] and access fields after
/// [`crate::init_metrics_impl`] has completed successfully. Access before
/// startup completion panics because direct field access has no error channel.
pub struct MetricsCell<T> {
    metrics: OnceLock<T>,
    init_lock: Mutex<()>,
}

impl<T> MetricsCell<T> {
    /// Create an empty metrics cell for a `static` item.
    pub const fn new() -> Self {
        Self {
            metrics: OnceLock::new(),
            init_lock: Mutex::new(()),
        }
    }

    /// Return initialized metrics, if startup has completed.
    pub fn get(&self) -> Option<&T> {
        self.metrics.get()
    }

    /// Initialize metrics once.
    pub fn init(&self, name: &str, init: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<()> {
        let _guard = self
            .init_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("metrics initializer lock poisoned: {name}"))?;

        if self.get().is_some() {
            return Ok(());
        }

        self.metrics
            .set(init()?)
            .map_err(|_| anyhow::anyhow!("metrics already initialized: {name}"))
    }
}

impl<T> Default for MetricsCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MetricsCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.get() {
            Some(metrics) => metrics,
            None => panic!("metrics used before initialization: {}", std::any::type_name::<T>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetricsCell;

    #[test]
    fn test_init_keeps_cell_empty_on_error() {
        let metrics = MetricsCell::<u8>::new();

        let result = metrics.init("test metrics", || anyhow::bail!("init failed"));

        assert!(result.is_err());
        assert!(metrics.get().is_none());
    }

    #[test]
    fn test_init_is_idempotent() -> anyhow::Result<()> {
        let metrics = MetricsCell::<u8>::new();

        metrics.init("test metrics", || Ok(7))?;
        metrics.init("test metrics", || Ok(9))?;

        assert_eq!(*metrics, 7);
        Ok(())
    }
}
