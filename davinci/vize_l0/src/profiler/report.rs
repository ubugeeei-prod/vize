//! Summary and entry types produced from collected profile data.

use std::time::Duration;

/// A summary of profiling data.
#[derive(Debug)]
pub struct ProfileSummary {
    /// Entries sorted by total time
    pub entries: Vec<ProfileEntry>,
}

impl ProfileSummary {
    /// Check if any operation exceeded the threshold.
    pub fn has_slow_operations(&self, threshold: Duration) -> bool {
        self.entries.iter().any(|e| e.average > threshold)
    }

    /// Get slow operations.
    pub fn slow_operations(&self, threshold: Duration) -> Vec<&ProfileEntry> {
        self.entries
            .iter()
            .filter(|e| e.average > threshold)
            .collect()
    }
}

/// A summary of non-duration profile counters.
#[derive(Debug)]
pub struct CounterSummary {
    /// Counter entries sorted by name.
    pub entries: Vec<CounterEntry>,
}

impl CounterSummary {
    /// Get a counter total by name.
    pub fn total(&self, name: &str) -> u64 {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.total)
            .unwrap_or(0)
    }

    /// Sum counter totals matching a prefix and suffix.
    pub fn total_matching(&self, prefix: &str, suffix: &str) -> u64 {
        self.entries
            .iter()
            .filter(|entry| entry.name.starts_with(prefix) && entry.name.ends_with(suffix))
            .map(|entry| entry.total)
            .sum()
    }
}

/// A single non-duration counter entry.
#[derive(Debug)]
pub struct CounterEntry {
    /// Counter name.
    pub name: &'static str,
    /// Number of samples recorded.
    pub samples: u64,
    /// Sum of all recorded samples.
    pub total: u64,
    /// Average value per sample.
    pub average: f64,
    /// Smallest sample.
    pub min: u64,
    /// Largest sample.
    pub max: u64,
}

impl std::fmt::Display for ProfileSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Profile Summary:")?;
        writeln!(
            f,
            "{:<30} {:>8} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "Operation", "Count", "Total ms", "Self ms", "Avg ms", "P95 ms", "Min ms", "Max ms"
        )?;
        writeln!(f, "{}", "-".repeat(114))?;

        for entry in &self.entries {
            writeln!(
                f,
                "{:<30} {:>8} {:>12.3} {:>12.3} {:>12.3} {:>12.3} {:>12.3} {:>12.3}",
                entry.name,
                entry.count,
                duration_ms(entry.total),
                duration_ms(entry.self_total),
                duration_ms(entry.average),
                duration_ms(entry.p95),
                duration_ms(entry.min),
                duration_ms(entry.max)
            )?;
        }

        Ok(())
    }
}

#[inline]
fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// A single entry in the profile summary.
#[derive(Debug)]
pub struct ProfileEntry {
    /// Operation name
    pub name: &'static str,
    /// Number of calls
    pub count: u64,
    /// Total duration
    pub total: Duration,
    /// Total self duration excluding nested child spans
    pub self_total: Duration,
    /// Total nested child duration
    pub child_total: Duration,
    /// Average duration
    pub average: Duration,
    /// Average self duration
    pub self_average: Duration,
    /// Minimum duration
    pub min: Duration,
    /// Maximum duration
    pub max: Duration,
    /// Minimum self duration
    pub self_min: Duration,
    /// Maximum self duration
    pub self_max: Duration,
    /// Approximate p50 duration
    pub p50: Duration,
    /// Approximate p95 duration
    pub p95: Duration,
    /// Approximate p99 duration
    pub p99: Duration,
    /// Samples at or above 1ms
    pub samples_over_1ms: u64,
    /// Samples at or above 10ms
    pub samples_over_10ms: u64,
    /// Samples at or above 100ms
    pub samples_over_100ms: u64,
}
