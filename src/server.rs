use crate::base_metrics::update_base_metrics;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use prometheus::Encoder;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Handle for a running `/metrics` HTTP server.
///
/// Dropping the handle signals shutdown. Use [`MetricsServer::stop`] when the
/// caller needs to wait until the server task exits.
#[derive(Debug)]
pub struct MetricsServer {
    listen_address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<anyhow::Result<()>>>,
}

impl MetricsServer {
    /// Return the socket address selected by the listener.
    pub fn listen_address(&self) -> SocketAddr {
        self.listen_address
    }

    /// Signal shutdown and wait for the server task to finish.
    pub async fn stop(mut self) -> anyhow::Result<()> {
        self.shutdown();
        if let Some(task) = self.task.take() {
            task.await??;
        }
        Ok(())
    }

    pub(crate) async fn start(listen_address: &str) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(listen_address).await?;
        let listen_address = listener.local_addr()?;
        let (shutdown, shutdown_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            let router = Router::new().route("/metrics", get(metrics_handler));
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await?;
            Ok(())
        });

        tracing::info!("MetricServer is listening on http://{listen_address:?}/metrics");
        Ok(Self {
            listen_address,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    fn shutdown(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

impl Drop for MetricsServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

async fn metrics_handler() -> Result<String, StatusCode> {
    update_base_metrics();
    collect_metrics().map_err(|error| {
        tracing::error!("fail to collect metrics: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn collect_metrics() -> anyhow::Result<String> {
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&prometheus::gather(), &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{CacheStatsMetric, init_metrics};
    use prometheus::CounterVec;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_server() -> anyhow::Result<()> {
        let metrics_server = init_metrics!("127.0.0.1:0").await?;
        let port = metrics_server.listen_address().port();

        let counter = CounterVec::new(
            prometheus::Opts::new("test_counter", "test counter help"),
            &["label1", "label2"],
        )?;
        prometheus::register(Box::new(counter.clone()))?;
        counter.with_label_values(&["val1", "val2"]).inc_by(5.0);
        CacheStatsMetric::inc_request("test_cache");
        CacheStatsMetric::inc_miss("test_cache");

        let resp = loop {
            let resp = reqwest::get(format!("http://localhost:{port}/metrics")).await;
            if resp.is_err() {
                sleep(Duration::from_millis(20)).await;
                continue;
            }
            break resp?;
        };

        assert_eq!(resp.status(), 200);
        let text = resp.text().await?;
        assert!(text.contains("# HELP test_counter test counter help\n#"));
        assert!(text.contains("# TYPE stonfi_metrics_uptime_seconds"));
        assert!(text.contains("version"));
        assert!(text.contains("author"));
        assert!(text.contains("commit"));
        assert!(text.contains(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")));
        assert!(text.contains(option_env!("GITLAB_USER_EMAIL").unwrap_or("local-dev")));
        assert!(text.contains(option_env!("CI_COMMIT_SHORT_SHA").unwrap_or("local")));
        assert!(text.contains("# TYPE stonfi_metrics_cache_stats_total counter"));
        assert!(text.contains("stonfi_metrics_cache_stats_total{cache_name=\"test_cache\",result=\"request\"} 1"));
        assert!(text.contains("stonfi_metrics_cache_stats_total{cache_name=\"test_cache\",result=\"miss\"} 1"));

        metrics_server.stop().await?;
        Ok(())
    }
}
