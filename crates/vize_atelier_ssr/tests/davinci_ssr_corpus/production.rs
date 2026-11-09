//! The production sweep: every SFC compiled through the adapter entry point
//! exactly as the Vite plugin calls it in SSR mode, once on the selected lane
//! and once with the legacy walker pinned
//! (`vize_atelier_sfc::ssr_differential`, shared with the reducer's divergence
//! predicate). The whole SFC output must be byte-identical, and each
//! production SSR compile's lane verdict is the honest production-reach
//! number: unlike the emitter sweep, nothing here is derived by hand, so a
//! Croquis summary or any other option the SFC compiler adds reaches the
//! selector as it does in production.

use std::collections::BTreeMap;

use vize_atelier_sfc::ssr_differential::compare_production;
use vize_atelier_ssr::differential::on_both_lanes;

#[derive(Default)]
pub(crate) struct ProductionReport {
    pub(crate) templates: u64,
    pub(crate) compared: u64,
    pub(crate) error_skips: u64,
    /// Lane verdicts of every production SSR compile (`s4`, `legacy.<reason>`,
    /// `rejected`); `none` when an SFC with a template never reached SSR.
    pub(crate) lanes: BTreeMap<&'static str, u64>,
    pub(crate) rejected: Vec<String>,
    pub(crate) divergences: Vec<String>,
}

impl ProductionReport {
    pub(crate) fn reach(&self) -> u64 {
        self.lanes.get("s4").copied().unwrap_or_default()
    }
}

pub(crate) fn compare(name: &str, source: &str, report: &mut ProductionReport) {
    let Some(pair) = compare_production(name, source, |compile| on_both_lanes(compile)) else {
        return;
    };
    report.templates += 1;
    let verdicts = &pair.verdicts;
    if verdicts.is_empty() {
        *report.lanes.entry("none").or_default() += 1;
    }
    for verdict in verdicts {
        *report.lanes.entry(verdict).or_default() += 1;
        if *verdict == "rejected" {
            report.rejected.push(name.to_owned());
        }
    }
    let Some(differs) = pair.differs() else {
        report.error_skips += 1;
        return;
    };
    report.compared += 1;
    if !differs {
        return;
    }
    let detail = match (&pair.selected, &pair.legacy) {
        (Err(error), _) => format!(": selected lane failed: {error:?}"),
        (Ok(selected), Ok(legacy)) if report.divergences.len() < 10 => format!(
            "\n--- selected\n{}\n--- legacy\n{}",
            selected.code, legacy.code
        ),
        _ => String::new(),
    };
    report
        .divergences
        .push(format!("{name} {verdicts:?}{detail}"));
}

pub(crate) fn assert_clean(label: &str, report: &ProductionReport) {
    assert!(
        report.rejected.is_empty(),
        "{label}: broken L4 plan invariants on the production path: {:#?}",
        report.rejected
    );
    assert!(
        report.divergences.is_empty(),
        "{label}: {} production SFC divergences:\n{}",
        report.divergences.len(),
        report.divergences.join("\n")
    );
}
