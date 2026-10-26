//! Optional candidate inventory and actual authored-presence search observations.

use std::time::{Duration, Instant};

use vize_carton::profiler::{Profiler, global_profiler};

#[derive(Default)]
struct SearchMetrics {
    quoted_candidates: u64,
    eligible_candidates: u64,
    searches: u64,
    input_bytes: u64,
    time: Duration,
}

pub(super) struct SearchObservation<'a> {
    profiler: &'a Profiler,
    metrics: Option<SearchMetrics>,
}

impl SearchObservation<'static> {
    pub(super) fn new() -> Self {
        Self::for_profiler(global_profiler())
    }
}

impl<'a> SearchObservation<'a> {
    fn for_profiler(profiler: &'a Profiler) -> Self {
        Self {
            profiler,
            metrics: profiler.is_enabled().then(SearchMetrics::default),
        }
    }

    pub(super) fn candidate(&mut self, eligible: bool) {
        if let Some(metrics) = &mut self.metrics {
            metrics.quoted_candidates += 1;
            metrics.eligible_candidates += u64::from(eligible);
        }
    }

    pub(super) fn source_contains(&mut self, source: &str, reported: &str) -> bool {
        let Some(metrics) = &mut self.metrics else {
            return source.contains(reported);
        };
        metrics.searches += 1;
        // Haystack input volume, not a claim about bytes read by `contains`.
        metrics.input_bytes += source.len() as u64;
        let start = Instant::now();
        let present = source.contains(reported);
        metrics.time = metrics.time.saturating_add(start.elapsed());
        present
    }

    pub(super) fn finish(self, message: &str, rewrites: usize) {
        let Some(metrics) = self.metrics else {
            return;
        };
        // Aggregate locally; no profiler lock is taken per quoted candidate.
        for (key, value) in [
            ("canon.diagnostics.restore_specifiers.calls", 1),
            (
                "canon.diagnostics.restore_specifiers.message_input_bytes",
                message.len() as u64,
            ),
            (
                "canon.diagnostics.restore_specifiers.quoted_candidates",
                metrics.quoted_candidates,
            ),
            (
                "canon.diagnostics.restore_specifiers.eligible_candidates",
                metrics.eligible_candidates,
            ),
            (
                "canon.diagnostics.restore_specifiers.source_searches",
                metrics.searches,
            ),
            (
                "canon.diagnostics.restore_specifiers.source_search_input_bytes",
                metrics.input_bytes,
            ),
            (
                "canon.diagnostics.restore_specifiers.source_search_ns",
                u64::try_from(metrics.time.as_nanos()).unwrap_or(u64::MAX),
            ),
            (
                "canon.diagnostics.restore_specifiers.rewrites",
                rewrites as u64,
            ),
        ] {
            self.profiler.record_counter_enabled(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_observation_retains_presence_results_and_utf8_input_volume() {
        let profiler = Profiler::enabled();
        let source = "import './実在.vue.ts'; const label = '🙂';";
        let mut observation = SearchObservation::for_profiler(&profiler);
        observation.candidate(true);
        assert!(observation.source_contains(source, "./実在.vue.ts"));
        observation.candidate(true);
        assert!(!observation.source_contains(source, "./Missing.vue.ts"));
        observation.candidate(false);
        observation.finish("full message 🙂", 1);
        let mut counters = profiler.counter_snapshot();
        counters.sort_by_key(|(key, _)| *key);
        assert_eq!(counters.len(), 8);
        for (key, metrics) in counters {
            assert_eq!(metrics.samples, 1);
            let suffix = key
                .strip_prefix("canon.diagnostics.restore_specifiers.")
                .unwrap();
            let expected = match suffix {
                "calls" | "rewrites" => 1,
                "message_input_bytes" => "full message 🙂".len() as u64,
                "quoted_candidates" => 3,
                "eligible_candidates" | "source_searches" => 2,
                "source_search_input_bytes" => 2 * source.len() as u64,
                "source_search_ns" => continue,
                _ => panic!("unexpected metric {key}"),
            };
            assert_eq!(metrics.total, expected, "{key}");
        }
    }

    #[test]
    fn disabled_observation_only_performs_the_existing_search() {
        let profiler = Profiler::new();
        let mut observation = SearchObservation::for_profiler(&profiler);
        assert!(observation.metrics.is_none());
        observation.candidate(false);
        assert!(observation.source_contains("import './Panel.vue.ts';", "./Panel.vue.ts"));
        assert!(!observation.source_contains("🙂", "absent"));
        observation.finish("unchanged", 0);
        assert!(profiler.counter_snapshot().is_empty());
    }

    #[test]
    fn ineligible_inventory_does_not_report_a_source_search() {
        let profiler = Profiler::enabled();
        let mut observation = SearchObservation::for_profiler(&profiler);
        for _ in 0..3 {
            observation.candidate(false);
        }
        observation.finish("Type 'number' differs from 'string' and 'undefined'.", 0);
        let counters = profiler.counter_snapshot();
        assert_eq!(counters.len(), 8);
        for (key, metrics) in counters {
            assert_eq!(metrics.samples, 1);
            if key.ends_with(".calls") {
                assert_eq!(metrics.total, 1);
            } else if key.ends_with(".quoted_candidates") {
                assert_eq!(metrics.total, 3);
            } else if !key.ends_with(".message_input_bytes") {
                assert_eq!(metrics.total, 0, "{key}");
            }
        }
    }
}
