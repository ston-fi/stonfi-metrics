use std::ops::Deref;
use std::sync::OnceLock;

/// Fallibly initialized storage for module-owned metrics.
///
/// Register this with [`crate::register_metrics!`] and access fields after
/// [`crate::init_metrics_impl`] has completed successfully. Access before
/// startup completion panics because direct field access has no error channel.
pub struct MetricsCell<T>(OnceLock<T>);

impl<T> MetricsCell<T> {
    /// Create an empty metrics cell for a `static` item.
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    /// Return initialized metrics, if startup has completed.
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }

    /// Store initialized metrics.
    pub fn set(&self, metrics: T) -> Result<(), T> {
        self.0.set(metrics)
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
            None => panic!(
                "metrics used before initialization: {}",
                std::any::type_name::<T>()
            ),
        }
    }
}
