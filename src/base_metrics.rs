use crate::MetricsCell;
use prometheus::{GaugeVec, register_gauge_vec};
use std::time::Instant;

static BASE_METRICS: MetricsCell<BaseMetrics> = MetricsCell::new();

pub(super) fn init_base_metrics(version: &str, commit: &str, author: &str) -> anyhow::Result<()> {
    BASE_METRICS.init("base metrics", || BaseMetrics::new(version, commit, author))
}

pub(super) fn update_base_metrics() {
    let Some(base) = BASE_METRICS.get() else {
        return;
    };
    base.update();
}

#[derive(Debug)]
struct BaseMetrics {
    start_time: Instant,
    label_values: [String; 3],
    uptime_sec: GaugeVec,
}

impl BaseMetrics {
    fn new(version: &str, commit: &str, author: &str) -> anyhow::Result<Self> {
        Ok(Self {
            start_time: Instant::now(),
            label_values: [version.to_string(), commit.to_string(), author.to_string()],
            uptime_sec: register_gauge_vec!(
                "stonfi_metrics_uptime_seconds",
                "Uptime of the service",
                &["version", "commit", "author"]
            )?,
        })
    }

    fn update(&self) {
        let label_values = self.label_values.each_ref().map(String::as_str);
        self.uptime_sec
            .with_label_values(&label_values)
            .set(self.start_time.elapsed().as_secs_f64());
    }
}
