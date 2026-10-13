//! Typed attribution for DOM templates kept off the L2 emitter.
//!
//! The selection counters name one `legacy.emit_refused` lane. A corpus sweep
//! needs the gate behind each refusal to pick the next installment from
//! counts, so the DOM compiler notes L2's [`EmitError`] on the differential
//! thread-local and this census groups the templates by
//! `UnsupportedReason::code()`. A template with no note never reached the
//! emitter: the SFC selector kept the compatibility lane on its own.

use std::collections::BTreeMap;

use vize_l1_to_l2::EmitError;

/// L2 reported an error diagnostic and left the module to the shipped lane.
pub const DIAGNOSTICS: &str = "diagnostics";
/// The SFC selector kept the compatibility lane without running L2.
pub const SELECTOR: &str = "selector";

/// Files kept per bucket unless `VIZE_DAVINCI_REACH_REFUSAL_SAMPLES` raises it.
const DEFAULT_SAMPLES: usize = 3;

/// The stable bucket of one refusal and the refused template byte offset.
pub fn bucket(refusal: Option<EmitError>) -> (&'static str, Option<u32>) {
    match refusal {
        Some(EmitError::Unsupported(refusal)) => {
            (refusal.reason.code(), refusal.span.map(|span| span.start))
        }
        Some(EmitError::Diagnostics) => (DIAGNOSTICS, None),
        None => (SELECTOR, None),
    }
}

/// `emit_refused` templates of one DOM shape, per typed bucket.
#[derive(Debug)]
pub struct RefusalCensus {
    counts: BTreeMap<&'static str, u64>,
    samples: BTreeMap<&'static str, Vec<String>>,
    sample_limit: usize,
}

impl Default for RefusalCensus {
    fn default() -> Self {
        Self::with_sample_limit(
            std::env::var("VIZE_DAVINCI_REACH_REFUSAL_SAMPLES")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(DEFAULT_SAMPLES),
        )
    }
}

impl RefusalCensus {
    /// A census keeping at most `sample_limit` files per bucket.
    pub fn with_sample_limit(sample_limit: usize) -> Self {
        Self {
            counts: BTreeMap::new(),
            samples: BTreeMap::new(),
            sample_limit,
        }
    }

    /// Attribute one `emit_refused` template to its bucket.
    pub fn note(&mut self, name: &str, refusal: Option<EmitError>) {
        let (code, offset) = bucket(refusal);
        *self.counts.entry(code).or_default() += 1;
        let samples = self.samples.entry(code).or_default();
        if samples.len() < self.sample_limit {
            samples.push(match offset {
                Some(offset) => format!("{name}@{offset}"),
                None => name.to_owned(),
            });
        }
    }

    /// Every attributed template.
    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Templates attributed to `code`.
    pub fn count(&self, code: &str) -> u64 {
        self.counts.get(code).copied().unwrap_or(0)
    }

    /// The census line, then one sample line per bucket, largest first.
    pub fn lines(&self, scope: &str, shape: &str) -> Vec<String> {
        if self.counts.is_empty() {
            return Vec::new();
        }
        let mut buckets: Vec<(&'static str, u64)> = self
            .counts
            .iter()
            .map(|(code, count)| (*code, *count))
            .collect();
        buckets.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
        let mut lines = vec![format!(
            "davinci production reach refusals: scope={scope} shape={shape} total={} reasons={buckets:?}",
            self.total()
        )];
        for (code, _) in buckets {
            lines.push(format!(
                "davinci production reach refusal samples: scope={scope} shape={shape} reason={code} files={:?}",
                self.samples.get(code).map(Vec::as_slice).unwrap_or(&[])
            ));
        }
        lines
    }
}
