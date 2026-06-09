//! Per-operation duration metrics and non-duration counters.

use std::time::Duration;

const PROFILE_HISTOGRAM_BUCKETS: usize = 48;

/// Profiling metrics for a single operation.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Number of times this operation was called
    pub count: u64,
    /// Total duration across all calls
    pub total_duration: Duration,
    /// Total duration excluding nested child spans
    pub self_duration: Duration,
    /// Total nested child span duration
    pub child_duration: Duration,
    /// Minimum duration
    pub min_duration: Duration,
    /// Maximum duration
    pub max_duration: Duration,
    /// Minimum self duration
    pub min_self_duration: Duration,
    /// Maximum self duration
    pub max_self_duration: Duration,
    pub(super) histogram: [u64; PROFILE_HISTOGRAM_BUCKETS],
    pub(super) samples_over_1ms: u64,
    pub(super) samples_over_10ms: u64,
    pub(super) samples_over_100ms: u64,
}

impl Metrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            self_duration: Duration::ZERO,
            child_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            min_self_duration: Duration::MAX,
            max_self_duration: Duration::ZERO,
            histogram: [0; PROFILE_HISTOGRAM_BUCKETS],
            samples_over_1ms: 0,
            samples_over_10ms: 0,
            samples_over_100ms: 0,
        }
    }

    /// Record a duration.
    pub fn record(&mut self, duration: Duration) {
        self.record_with_child(duration, Duration::ZERO);
    }

    /// Record a duration and the already-accounted nested child span duration.
    pub fn record_with_child(&mut self, duration: Duration, child_duration: Duration) {
        let self_duration = duration.saturating_sub(child_duration);

        self.count += 1;
        self.total_duration += duration;
        self.self_duration += self_duration;
        self.child_duration += child_duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
        self.min_self_duration = self.min_self_duration.min(self_duration);
        self.max_self_duration = self.max_self_duration.max(self_duration);
        self.histogram[duration_bucket(duration)] += 1;

        if duration >= Duration::from_millis(1) {
            self.samples_over_1ms += 1;
        }
        if duration >= Duration::from_millis(10) {
            self.samples_over_10ms += 1;
        }
        if duration >= Duration::from_millis(100) {
            self.samples_over_100ms += 1;
        }
    }

    /// Get the average duration.
    pub fn average(&self) -> Duration {
        average_duration(self.total_duration, self.count)
    }

    /// Get the average self duration.
    pub fn self_average(&self) -> Duration {
        average_duration(self.self_duration, self.count)
    }

    /// Estimate a percentile from the logarithmic duration histogram.
    pub fn percentile(&self, percentile: f64) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }

        let target = ((self.count as f64) * percentile.clamp(0.0, 1.0)).ceil() as u64;
        let target = target.max(1);
        let mut seen = 0u64;
        for (index, count) in self.histogram.iter().enumerate() {
            seen += count;
            if seen >= target {
                return bucket_upper_bound(index);
            }
        }
        bucket_upper_bound(PROFILE_HISTOGRAM_BUCKETS - 1)
    }

    /// Number of samples at or above one millisecond.
    pub fn samples_over_1ms(&self) -> u64 {
        self.samples_over_1ms
    }

    /// Number of samples at or above ten milliseconds.
    pub fn samples_over_10ms(&self) -> u64 {
        self.samples_over_10ms
    }

    /// Number of samples at or above one hundred milliseconds.
    pub fn samples_over_100ms(&self) -> u64 {
        self.samples_over_100ms
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Monotonic counter metrics for non-duration profile signals.
#[derive(Debug, Clone)]
pub struct CounterMetrics {
    /// Number of samples recorded for this counter.
    pub samples: u64,
    /// Sum of all recorded values.
    pub total: u64,
    /// Smallest recorded value.
    pub min: u64,
    /// Largest recorded value.
    pub max: u64,
}

impl CounterMetrics {
    /// Create new counter metrics.
    pub fn new() -> Self {
        Self {
            samples: 0,
            total: 0,
            min: u64::MAX,
            max: 0,
        }
    }

    /// Record a counter sample.
    pub fn record(&mut self, value: u64) {
        self.samples = self.samples.saturating_add(1);
        self.total = self.total.saturating_add(value);
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    /// Get the average recorded value.
    pub fn average(&self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.total as f64 / self.samples as f64
        }
    }
}

impl Default for CounterMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn average_duration(duration: Duration, count: u64) -> Duration {
    if count == 0 {
        Duration::ZERO
    } else {
        let nanos = duration.as_nanos() / u128::from(count);
        Duration::from_nanos(nanos.try_into().unwrap_or(u64::MAX))
    }
}

#[inline]
fn duration_bucket(duration: Duration) -> usize {
    let mut upper_micros = 1u128;
    let micros = duration.as_micros();
    let mut bucket = 0usize;
    while bucket + 1 < PROFILE_HISTOGRAM_BUCKETS && micros > upper_micros {
        upper_micros <<= 1;
        bucket += 1;
    }
    bucket
}

#[inline]
fn bucket_upper_bound(bucket: usize) -> Duration {
    if bucket >= 63 {
        return Duration::MAX;
    }
    Duration::from_micros(1u64 << bucket)
}
