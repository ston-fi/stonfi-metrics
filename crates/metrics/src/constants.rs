use std::sync::LazyLock;

/// Exponential duration buckets in milliseconds from 1ms to about 20s.
pub static DURATION_BUCKETS_1MS_20S: LazyLock<Vec<f64>> =
    LazyLock::new(|| exponential_buckets(1.0, 3.01, 10)); // 1*(3.01^9) = 20281
/// Exponential duration buckets in milliseconds from 0.1ms to about 20s.
pub static DURATION_BUCKETS_01MS_20S: LazyLock<Vec<f64>> =
    LazyLock::new(|| exponential_buckets(0.1, 3.89, 10)); // 0.1*(3.89^9) = 20396

fn exponential_buckets(start: f64, factor: f64, count: usize) -> Vec<f64> {
    let mut buckets = Vec::with_capacity(count);
    let mut next = start;
    for _ in 0..count {
        buckets.push(next);
        next *= factor;
    }
    buckets
}
