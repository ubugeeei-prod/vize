//! Per-shape lane tallies and the `[reach]` budget floors.

use std::collections::BTreeMap;

use vize_s0::profiler::CounterSummary;

use super::shapes::Shape;

/// The lane one template compile took, read from its selection counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lane {
    /// The Davinci stage emitted the module.
    Accepted,
    /// The legacy lane emitted it, for the named reason.
    Legacy(String),
    /// The Davinci artifact broke an invariant and the legacy lane emitted.
    Rejected,
    /// The backend recorded no selection counter at all: its lane has no
    /// production selector yet (a witness-only bridge, for instance).
    Unrecorded,
}

/// Classify one compile from the counters it recorded, enforcing the
/// one-selection-per-template law.
pub fn classify(shape: Shape, counters: &CounterSummary) -> Result<Lane, String> {
    let namespace = shape.namespace();
    let mut lanes = Vec::new();
    for entry in &counters.entries {
        let Some(name) = entry.name.strip_prefix(namespace) else {
            continue;
        };
        let lane = if name == "accepted" {
            Lane::Accepted
        } else if name == "rejected" {
            Lane::Rejected
        } else if let Some(reason) = name.strip_prefix("legacy.") {
            Lane::Legacy(reason.to_owned())
        } else {
            continue;
        };
        for _ in 0..entry.total {
            lanes.push(lane.clone());
        }
    }
    match lanes.len() {
        0 => Ok(Lane::Unrecorded),
        1 => Ok(lanes.remove(0)),
        _ => Err(format!(
            "{} recorded {} selection counters for one template: {lanes:?}",
            shape.id(),
            lanes.len()
        )),
    }
}

/// One shape's tally over a corpus.
#[derive(Debug, Default)]
pub struct Tally {
    /// Templates the shape compiled successfully.
    pub templates: u64,
    /// Templates whose module the Davinci stage emitted.
    pub accepted: u64,
    /// Legacy-lane templates per reason.
    pub legacy: BTreeMap<String, u64>,
    /// Artifact rejections (the legacy lane emitted).
    pub rejected: u64,
    /// Templates with no selection counter.
    pub unrecorded: u64,
    /// The first few unrecorded template files.
    pub unrecorded_samples: Vec<String>,
    /// `compile_sfc` errors per error code (not templates this shape reached).
    pub sfc_errors: BTreeMap<String, u64>,
    /// Croquis-refused DOM templates S2 emits under the projection
    /// (`with_croquis_projection`): parity-ready, not yet selected.
    pub ready: u64,
    /// DOM templates (accepted or parity-ready) compared against the forced
    /// legacy lane.
    pub compared: u64,
    /// Byte divergences between the selected and the forced legacy lane.
    pub divergences: Vec<String>,
    /// Selection-law violations.
    pub violations: Vec<String>,
}

impl Tally {
    pub fn record(&mut self, lane: Lane) {
        self.templates += 1;
        match lane {
            Lane::Accepted => self.accepted += 1,
            Lane::Legacy(reason) => *self.legacy.entry(reason).or_default() += 1,
            Lane::Rejected => self.rejected += 1,
            Lane::Unrecorded => self.unrecorded += 1,
        }
    }

    /// Accepted templates per thousand compiled templates (floor).
    pub fn permille(&self) -> u64 {
        (self.accepted * 1000)
            .checked_div(self.templates)
            .unwrap_or(0)
    }

    pub fn line(&self, shape: Shape) -> String {
        format!(
            "davinci production reach: shape={} stage={:?} templates={} accepted={} permille={} ready={} legacy={:?} rejected={} unrecorded={} sfc_errors={:?} compared={} divergences={}",
            shape.id(),
            shape.stage(),
            self.templates,
            self.accepted,
            self.permille(),
            self.ready,
            self.legacy,
            self.rejected,
            self.unrecorded,
            self.sfc_errors,
            self.compared,
            self.divergences.len(),
        )
    }
}

/// The committed `[reach]` floor for one shape.
#[derive(Debug, Clone, Copy)]
pub struct Floor {
    pub accepted_min: u64,
    pub ready_min: u64,
    pub templates_min: u64,
}

/// Read the `[reach]` section of `docs/davinci/plan/reach-budgets.toml`.
pub fn floors() -> BTreeMap<String, Floor> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/davinci/plan/reach-budgets.toml"
    );
    let text = std::fs::read_to_string(path).expect("read docs/davinci/plan/reach-budgets.toml");
    let budgets: toml::Table = text.parse().expect("parse reach-budgets.toml");
    let reach = budgets
        .get("reach")
        .and_then(toml::Value::as_table)
        .expect("reach-budgets.toml must carry a [reach] section");
    let field = |entry: &toml::Table, id: &str, key: &str| -> u64 {
        let value = entry
            .get(key)
            .and_then(toml::Value::as_integer)
            .unwrap_or_else(|| panic!("[reach].{id} must carry an integer `{key}`"));
        u64::try_from(value).unwrap_or_else(|_| panic!("[reach].{id}.{key} must be >= 0"))
    };
    let mut floors = BTreeMap::new();
    for (id, entry) in reach {
        let Some(entry) = entry.as_table() else {
            continue;
        };
        floors.insert(
            id.clone(),
            Floor {
                accepted_min: field(entry, id, "accepted_min"),
                ready_min: field(entry, id, "ready_min"),
                templates_min: field(entry, id, "templates_min"),
            },
        );
    }
    floors
}
