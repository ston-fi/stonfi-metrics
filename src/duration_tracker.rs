use crate::utils::format_duration_ms;

/// Timer that observes elapsed milliseconds into a `prometheus::Histogram`.
pub struct DurationTracker<'a> {
    metric: &'a prometheus::Histogram,
    start: std::time::Instant,
}

impl DurationTracker<'_> {
    /// Start a timer for `metric`.
    pub fn new(metric: &prometheus::Histogram) -> DurationTracker<'_> {
        DurationTracker {
            metric,
            start: std::time::Instant::now(),
        }
    }

    /// Return elapsed time without recording it.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Drop for DurationTracker<'_> {
    fn drop(&mut self) {
        self.metric
            .observe(format_duration_ms(self.start.elapsed()));
    }
}

/// Timer that observes elapsed milliseconds into a labeled histogram.
pub struct DurationTrackerVec<'a, 'b, 'c> {
    metric: &'a prometheus::HistogramVec,
    labels: &'b [&'c str],
    start: std::time::Instant,
}

impl DurationTrackerVec<'_, '_, '_> {
    /// Start a timer for `metric` with `labels`.
    pub fn new<'a, 'b, 'c>(
        metric: &'a prometheus::HistogramVec,
        labels: &'b [&'c str],
    ) -> DurationTrackerVec<'a, 'b, 'c> {
        DurationTrackerVec {
            metric,
            labels,
            start: std::time::Instant::now(),
        }
    }

    /// Return elapsed time without recording it.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Drop for DurationTrackerVec<'_, '_, '_> {
    fn drop(&mut self) {
        self.metric
            .with_label_values(self.labels)
            .observe(format_duration_ms(self.start.elapsed()));
    }
}

/// Track elapsed time and observe it into a histogram when the guard is dropped.
///
/// Supports both `prometheus::Histogram` and `prometheus::HistogramVec`.
#[macro_export]
macro_rules! track_duration {
    ($metric:expr) => {
        $crate::duration_tracker::DurationTracker::new(&$metric)
    };
    ($metric:expr, $labels:expr) => {
        $crate::duration_tracker::DurationTrackerVec::new(&$metric, $labels)
    };
}
