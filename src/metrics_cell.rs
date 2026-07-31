use std::ops::Deref;
use std::sync::{Mutex, OnceLock};

/// Fallibly initialized storage for module-owned metrics.
///
/// Register this with [`crate::register_metrics!`]. Accessing fields before
/// explicit metrics startup lazily initializes this cell and emits a warning.
/// Prefer [`crate::init_metrics_impl`] so initialization failures are returned
/// during startup.
///
/// Initializers run while holding the cell's initialization lock. They must not
/// access their own cell or create a cyclic dependency between metrics cells.
///
/// # Panics
///
/// Dereferencing an uninitialized cell panics when its registered initializer
/// fails or when the cell was not registered with [`crate::register_metrics!`].
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
    ///
    /// # Errors
    ///
    /// Returns an error when the initializer fails or the initialization lock
    /// is poisoned.
    pub fn init(&self, name: &str, init: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<()> {
        self.init_if_needed(name, init).map(|_| ())
    }

    pub(crate) fn init_if_needed(&self, name: &str, init: impl FnOnce() -> anyhow::Result<T>) -> anyhow::Result<bool> {
        let _guard = self
            .init_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("metrics initializer lock poisoned: {name}"))?;

        if self.get().is_some() {
            return Ok(false);
        }

        self.metrics
            .set(init()?)
            .map_err(|_| anyhow::anyhow!("metrics already initialized: {name}"))?;

        Ok(true)
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
            None => match crate::initializer::init_registered_metric(self) {
                Ok(metrics) => metrics,
                Err(error) => panic!("failed to initialize metrics on first use: {error:#}"),
            },
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
