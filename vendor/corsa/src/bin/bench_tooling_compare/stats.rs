use std::time::Duration;

use corsa::fast::SmallVec;

#[derive(Clone, Debug)]
pub struct Stats {
    sample_count: usize,
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    p95: Duration,
    p99: Duration,
    stddev: Duration,
}

impl Stats {
    pub fn from_samples(mut samples: SmallVec<[Duration; 32]>) -> Self {
        samples.sort_unstable();
        let sample_count = samples.len();
        if sample_count == 0 {
            return Self {
                sample_count,
                min: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                stddev: Duration::ZERO,
            };
        }
        let min = samples[0];
        let max = samples[samples.len() - 1];
        let median = samples[samples.len() / 2];
        let p95 = samples[percentile_index(samples.len(), 95)];
        let p99 = samples[percentile_index(samples.len(), 99)];
        let total_nanos = samples
            .iter()
            .fold(0_u128, |acc, sample| acc + sample.as_nanos());
        let mean_nanos = total_nanos as f64 / sample_count as f64;
        let mean = Duration::from_secs_f64(mean_nanos / 1_000_000_000.0);
        let variance_nanos = samples
            .iter()
            .map(|sample| {
                let delta = sample.as_nanos() as f64 - mean_nanos;
                delta * delta
            })
            .sum::<f64>()
            / sample_count as f64;
        let stddev = Duration::from_secs_f64(variance_nanos.sqrt() / 1_000_000_000.0);
        Self {
            sample_count,
            min,
            max,
            mean,
            median,
            p95,
            p99,
            stddev,
        }
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn mean_ms(&self) -> f64 {
        millis(self.mean)
    }

    pub fn median_ms(&self) -> f64 {
        millis(self.median)
    }

    pub fn p95_ms(&self) -> f64 {
        millis(self.p95)
    }

    pub fn p99_ms(&self) -> f64 {
        millis(self.p99)
    }

    pub fn stddev_ms(&self) -> f64 {
        millis(self.stddev)
    }

    pub fn cv_percent(&self) -> f64 {
        if self.mean.is_zero() {
            return 0.0;
        }
        (self.stddev.as_secs_f64() / self.mean.as_secs_f64()) * 100.0
    }

    pub fn min_ms(&self) -> f64 {
        millis(self.min)
    }

    pub fn max_ms(&self) -> f64 {
        millis(self.max)
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn percentile_index(len: usize, percentile: usize) -> usize {
    len.saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use corsa::fast::SmallVec;

    use super::Stats;

    #[test]
    fn stats_compute_expected_percentiles() {
        let mut samples = SmallVec::<[Duration; 32]>::new();
        samples.push(Duration::from_millis(9));
        samples.push(Duration::from_millis(1));
        samples.push(Duration::from_millis(5));
        samples.push(Duration::from_millis(7));
        samples.push(Duration::from_millis(3));
        let stats = Stats::from_samples(samples);
        assert_eq!(stats.sample_count(), 5);
        assert_eq!(stats.min_ms(), 1.0);
        assert_eq!(stats.median_ms(), 5.0);
        assert_eq!(stats.p95_ms(), 9.0);
        assert_eq!(stats.p99_ms(), 9.0);
        assert_eq!(stats.max_ms(), 9.0);
        assert!(stats.stddev_ms() > 0.0);
        assert!(stats.cv_percent() > 0.0);
    }

    #[test]
    fn stats_handle_empty_samples() {
        let stats = Stats::from_samples(SmallVec::<[Duration; 32]>::new());
        assert_eq!(stats.sample_count(), 0);
        assert_eq!(stats.median_ms(), 0.0);
        assert_eq!(stats.p95_ms(), 0.0);
    }
}
