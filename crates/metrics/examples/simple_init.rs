use std::sync::LazyLock;
use std::time::Duration;

use prometheus::{
    HistogramOpts, HistogramVec, IntCounterVec, register_histogram_vec, register_int_counter_vec,
};
use stonfi_metrics::constants::DURATION_BUCKETS_1MS_20S;
use stonfi_metrics::track_duration;

static METRICS: LazyLock<Option<Metrics>> = LazyLock::new(|| Metrics::new().ok());

struct Metrics {
    requests_total: IntCounterVec,
    request_duration: HistogramVec,
}

impl Metrics {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            requests_total: register_int_counter_vec!(
                "stonfi_example_requests_total",
                "Total number of handled requests",
                &["method"]
            )?,
            request_duration: register_histogram_vec!(
                HistogramOpts::new(
                    "stonfi_example_request_duration_ms",
                    "Request handling duration in milliseconds",
                )
                .buckets(DURATION_BUCKETS_1MS_20S.to_vec()),
                &["method"]
            )?,
        })
    }

    fn inc_requests(label: &str) {
        if let Some(metrics) = METRICS.as_ref() {
            metrics.requests_total.with_label_values(&[label]).inc();
        };
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let metrics_server = stonfi_metrics::init_metrics!("127.0.0.1:0").await?;

    Metrics::inc_requests("GET");
    if let Some(metrics) = METRICS.as_ref() {
        let _timer = track_duration!(metrics.request_duration, &["GET"]);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    println!(
        "metrics listening on http://{}/metrics",
        metrics_server.listen_address()
    );
    metrics_server.stop().await
}
