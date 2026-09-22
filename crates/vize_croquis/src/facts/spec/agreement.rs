//! [`Agreement`] — one TS-34 run's scope proof and verdict.

use std::collections::BTreeMap;

use vize_carton::{CompactString, append, cstr};

/// The tally of one spec-versus-production run over a file set.
///
/// Counts every artifact it saw, every artifact it compared, every skip by
/// named reason, and every fact compared; records every divergence. The
/// verdict fails on a divergence **and** on a run that compared nothing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Agreement {
    /// Artifacts offered to the spec.
    pub artifacts: u64,
    /// Artifacts inside the spec's scope and compared.
    pub compared: u64,
    /// Facts compared across every compared artifact.
    pub facts: u64,
    /// Artifacts outside the spec's scope, by reason.
    pub skipped: BTreeMap<&'static str, u64>,
    /// One line per diverging artifact.
    pub divergences: Vec<CompactString>,
}

impl Agreement {
    /// Record an artifact outside the spec's scope.
    pub fn skip(&mut self, reason: &'static str) {
        self.artifacts += 1;
        *self.skipped.entry(reason).or_insert(0) += 1;
    }

    /// Record a compared artifact: `facts` facts, and the divergence line
    /// when production and spec disagree.
    pub fn compare(&mut self, facts: usize, divergence: Option<CompactString>) {
        self.artifacts += 1;
        self.compared += 1;
        self.facts += facts as u64;
        if let Some(line) = divergence {
            self.divergences.push(line);
        }
    }

    /// Fold another run into this one.
    pub fn absorb(&mut self, other: Agreement) {
        self.artifacts += other.artifacts;
        self.compared += other.compared;
        self.facts += other.facts;
        for (reason, count) in other.skipped {
            *self.skipped.entry(reason).or_insert(0) += count;
        }
        self.divergences.extend(other.divergences);
    }

    /// The scope line a gate re-parses: `<group> <label>: artifacts=… compared=… facts=… skipped=…`.
    #[must_use]
    pub fn scope_line(&self, group: &str, label: &str) -> CompactString {
        let mut line = cstr!(
            "fact spec {group} {label}: artifacts={} compared={} facts={} skipped={}",
            self.artifacts,
            self.compared,
            self.facts,
            self.skipped.values().sum::<u64>()
        );
        for (reason, count) in &self.skipped {
            append!(line, " {reason}={count}");
        }
        line
    }

    /// Pass only with zero divergences and at least one compared fact.
    ///
    /// # Errors
    ///
    /// The divergence list, or the degenerate-run message.
    pub fn verdict(&self, group: &str, label: &str) -> Result<(), CompactString> {
        if !self.divergences.is_empty() {
            let mut message = cstr!(
                "fact spec {group} {label}: {} of {} compared artifacts diverge",
                self.divergences.len(),
                self.compared
            );
            for line in self.divergences.iter().take(20) {
                append!(message, "\n  {line}");
            }
            return Err(message);
        }
        if self.facts == 0 {
            return Err(cstr!(
                "fact spec {group} {label}: zero facts were compared — the run proves nothing \
                 ({} artifacts, {} compared); a degenerated suite must fail, not pass",
                self.artifacts,
                self.compared
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Agreement;
    use vize_carton::CompactString;

    #[test]
    fn a_zero_fact_run_fails_and_divergences_are_reported() {
        let mut run = Agreement::default();
        run.skip("parse-error");
        run.compare(0, None);
        assert_eq!(
            run.verdict("bindings", "unit"),
            Err(CompactString::new(
                "fact spec bindings unit: zero facts were compared — the run proves nothing \
                 (2 artifacts, 1 compared); a degenerated suite must fail, not pass"
            ))
        );
        run.compare(3, Some(CompactString::new("a.vue: x")));
        assert_eq!(
            run.verdict("bindings", "unit"),
            Err(CompactString::new(
                "fact spec bindings unit: 1 of 2 compared artifacts diverge\n  a.vue: x"
            ))
        );
        assert_eq!(
            run.scope_line("bindings", "unit"),
            "fact spec bindings unit: artifacts=3 compared=2 facts=3 skipped=1 parse-error=1"
        );
    }
}
