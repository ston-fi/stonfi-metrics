use std::time::Duration;

/// Convert a [`Duration`] to milliseconds.
pub fn format_duration_ms(duration: Duration) -> f64 {
    duration.as_micros() as f64 / 1000.0
}
