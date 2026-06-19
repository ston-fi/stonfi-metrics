use parking_lot::Mutex;
use prometheus::{GaugeVec, register_gauge_vec};
use std::time::Instant;

static BASE_METRICS: Mutex<Option<BaseMetrics>> = Mutex::new(None);

pub(super) fn init_base_metrics(version: &str, commit: &str, author: &str) -> anyhow::Result<()> {
    let mut base_metrics = BASE_METRICS.lock();
    if base_metrics.is_some() {
        return Ok(());
    }

    let metrics = BaseMetrics::new(version, commit, author)?;
    *base_metrics = Some(metrics);
    Ok(())
}

pub(super) fn update_base_metrics() {
    let lock = BASE_METRICS.lock();
    let Some(base) = lock.as_ref() else {
        return;
    };
    base.update();
}

#[derive(Debug)]
struct BaseMetrics {
    start_time: Instant,
    label_values: Vec<String>,
    uptime_sec: GaugeVec,
}

impl BaseMetrics {
    fn new(version: &str, commit: &str, author: &str) -> anyhow::Result<Self> {
        Ok(Self {
            start_time: Instant::now(),
            label_values: vec![version.to_string(), commit.to_string(), author.to_string()],
            uptime_sec: register_gauge_vec!(
                "stonfi_metrics_uptime_seconds",
                "Uptime of the service",
                &["version", "commit", "author"]
            )?,
        })
    }

    fn update(&self) {
        let label_values = self
            .label_values
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        self.uptime_sec
            .with_label_values(&label_values)
            .set(self.start_time.elapsed().as_secs_f64());
    }
}
