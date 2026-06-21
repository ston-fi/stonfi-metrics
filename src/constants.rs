use std::sync::LazyLock;

/// Exponential duration buckets in milliseconds from 1ms to about 20s.
pub static DURATION_BUCKETS_1MS_20S: LazyLock<Vec<f64>> =
    LazyLock::new(|| exponential_buckets(1.0, 3.01, 10)); // 1*(3.01^9) = 20281
/// Exponential duration buckets in milliseconds from 0.1ms to about 20s.
pub static DURATION_BUCKETS_01MS_20S: LazyLock<Vec<f64>> =
    LazyLock::new(|| exponential_buckets(0.1, 3.89, 10)); // 0.1*(3.89^9) = 20396
/// Exponential duration buckets in milliseconds from 1s to 2m.
pub static DURATION_BUCKETS_1S_2M: LazyLock<Vec<f64>> =
    LazyLock::new(|| exponential_buckets(1_000.0, 1.7022374401561589, 10));

fn exponential_buckets(start: f64, factor: f64, count: usize) -> Vec<f64> {
    let mut buckets = Vec::with_capacity(count);
    let mut next = start;
    for _ in 0..count {
        buckets.push(next);
        next *= factor;
    }
    buckets
}

#[cfg(test)]
mod tests {
    use super::DURATION_BUCKETS_1S_2M;

    #[test]
    fn test_duration_buckets_1s_2m_cover_expected_range() {
        let buckets = DURATION_BUCKETS_1S_2M.as_slice();

        assert_eq!(buckets.len(), 10);
        assert!((buckets[0] - 1_000.0).abs() < f64::EPSILON);
        assert!((buckets[9] - 120_000.0).abs() < 0.000001);

        for window in buckets.windows(2) {
            assert!(window[0] < window[1]);
        }
    }
}
