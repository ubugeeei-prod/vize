//! TS-42 (P5-9): incremental ≡ clean, from the first salsa-backed release —
//! the rustc 1.52.1 lesson, verification on from day one.
//!
//! [`EquivalenceReport::check_file`] opens a file in one long-lived
//! [`ResidentDatabase`] and runs every edit script against it. After the
//! open and after **every** step it reads every artifact the tier serves —
//! S0 keys, block positions, S1 keys and tokenizer findings, S2 pages and
//! their diagnostics — and compares them, exactly, with
//! [`compute_file_artifacts`] run from scratch on the same text and config.
//!
//! Scope is counted, never assumed: files, script runs, applied and skipped
//! steps, state comparisons and block comparisons. [`EquivalenceReport::verdict`]
//! fails a run that compared nothing as surely as one that found a mismatch.

use core::fmt::Write as _;

use vize_s0::String;

use crate::artifact::{BlockArtifacts, StageConfig, compute_file_artifacts};
use crate::db::{ResidentDatabase, SourceFile};
use crate::snapshot::cancel::CancelToken;
use crate::snapshot::{SnapshotStats, SnapshotTree, Stages};

mod script;

pub use script::{EditOp, EditScript, Step, Target, parse_script};

/// One incremental result that differed from the clean one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// The file.
    pub file: String,
    /// The script (`open` for the initial read).
    pub script: String,
    /// 1-based step within the script (0 for the state before its first step).
    pub step: u32,
    /// The first differing block and field.
    pub detail: String,
}

/// Counts and mismatches of a TS-42 run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EquivalenceReport {
    /// Files checked.
    pub files: u32,
    /// Script runs (one per file per script).
    pub script_runs: u32,
    /// Steps applied through the database.
    pub steps_applied: u32,
    /// Operations skipped because the file has no target block.
    pub ops_skipped: u32,
    /// File states compared (incremental against clean).
    pub comparisons: u32,
    /// Blocks compared across all states.
    pub blocks_compared: u32,
    /// The snapshot tree's accounting summed over every update (P5-5): the
    /// snapshot path runs beside the database and is compared too.
    pub snapshot: SnapshotStats,
    /// Every state whose incremental artifacts differed.
    pub mismatches: Vec<Mismatch>,
}

impl EquivalenceReport {
    /// Run every script against `text` in one long-lived database. Each
    /// script starts from the original text and the default configuration.
    pub fn check_file(&mut self, path: &str, text: &str, scripts: &[EditScript]) {
        let mut db = ResidentDatabase::default();
        let file = db.open(path, text);
        self.files += 1;
        let mut state = State {
            path,
            text: String::from(text),
            config: StageConfig::default(),
            snapshot: None,
        };
        self.advance_snapshot(&mut state);
        self.compare(&db, file, &state, "open", 0);
        for script in scripts {
            self.script_runs += 1;
            if state.text != text || state.config != StageConfig::default() {
                if state.text != text {
                    state.text = String::from(text);
                    db.edit(file, text);
                }
                if state.config != StageConfig::default() {
                    state.config = StageConfig::default();
                    db.configure(state.config);
                }
                self.advance_snapshot(&mut state);
            }
            self.compare(&db, file, &state, &script.name, 0);
            let mut step_number = 0;
            for op in &script.ops {
                let steps = op.steps(&state.text, text);
                if steps.is_empty() {
                    self.ops_skipped += 1;
                }
                for step in steps {
                    match step {
                        Step::Text(next) => {
                            db.edit(file, &next);
                            state.text = next;
                        }
                        Step::Configure(vue_version) => {
                            state.config = StageConfig { vue_version };
                            db.configure(state.config);
                        }
                    }
                    self.advance_snapshot(&mut state);
                    step_number += 1;
                    self.steps_applied += 1;
                    self.compare(&db, file, &state, &script.name, step_number);
                }
            }
        }
    }

    fn compare(
        &mut self,
        db: &ResidentDatabase,
        file: SourceFile,
        state: &State<'_>,
        script: &str,
        step: u32,
    ) {
        let served = db.file_artifacts(file);
        let clean = compute_file_artifacts(state.text.as_str(), state.config);
        self.comparisons += 1;
        self.blocks_compared += clean.len() as u32;
        let snapshot = state
            .snapshot
            .as_ref()
            .map(SnapshotTree::artifacts)
            .unwrap_or_default();
        for (path, artifacts) in [("", served), ("snapshot ", snapshot)] {
            if artifacts != clean {
                let mut detail = String::from(path);
                detail.push_str(&describe(&artifacts, &clean));
                self.mismatches.push(Mismatch {
                    file: String::from(state.path),
                    script: String::from(script),
                    step,
                    detail,
                });
            }
        }
    }

    /// Update the state's snapshot tree to its current text and config.
    fn advance_snapshot(&mut self, state: &mut State<'_>) {
        // A fresh root token is never cancelled, so the update always
        // completes; if it did not, the previous tree stays current.
        let Ok((tree, stats)) = SnapshotTree::update(
            state.snapshot.as_ref(),
            state.text.as_str(),
            state.config,
            &Stages::DEFAULT,
            CancelToken::root(),
        ) else {
            return;
        };
        self.snapshot += stats;
        state.snapshot = Some(tree);
    }

    /// `Ok` only when something was compared and nothing differed.
    pub fn verdict(&self) -> Result<(), String> {
        if self.files == 0 || self.steps_applied == 0 || self.blocks_compared == 0 {
            return Err(String::from(
                "TS-42 compared nothing: a zero-file, zero-step or zero-block run proves no equivalence",
            ));
        }
        if !self.mismatches.is_empty() {
            let mut message = String::from("TS-42: incremental artifacts differ from clean ones");
            for mismatch in &self.mismatches {
                let _ = write!(
                    message,
                    "\n  {} [{} step {}] {}",
                    mismatch.file, mismatch.script, mismatch.step, mismatch.detail
                );
            }
            return Err(message);
        }
        Ok(())
    }

    /// The counts as `key=value` lines, then one `mismatch` line each.
    #[must_use]
    pub fn summary(&self) -> String {
        let mut out = String::default();
        for (key, value) in [
            ("files", self.files),
            ("script_runs", self.script_runs),
            ("steps_applied", self.steps_applied),
            ("ops_skipped", self.ops_skipped),
            ("comparisons", self.comparisons),
            ("blocks_compared", self.blocks_compared),
            ("snapshot_blocks_adopted", self.snapshot.blocks.adopted),
            ("snapshot_blocks_computed", self.snapshot.blocks.computed),
            ("snapshot_regions_adopted", self.snapshot.regions.adopted),
            ("snapshot_regions_computed", self.snapshot.regions.computed),
            ("mismatches", self.mismatches.len() as u32),
        ] {
            let _ = writeln!(out, "{key}={value}");
        }
        for mismatch in &self.mismatches {
            let _ = writeln!(
                out,
                "mismatch {} {} step={} {}",
                mismatch.file, mismatch.script, mismatch.step, mismatch.detail
            );
        }
        out
    }
}

struct State<'a> {
    path: &'a str,
    text: String,
    config: StageConfig,
    snapshot: Option<SnapshotTree>,
}

/// The first differing block and field.
fn describe(served: &[BlockArtifacts], clean: &[BlockArtifacts]) -> String {
    let mut out = String::default();
    if served.len() != clean.len() {
        let _ = write!(out, "block count {} != {}", served.len(), clean.len());
        return out;
    }
    for (served, clean) in served.iter().zip(clean) {
        let field = if served.kind != clean.kind || served.ordinal != clean.ordinal {
            "identity"
        } else if served.start != clean.start {
            "start"
        } else if served.source_key != clean.source_key {
            "s0-key"
        } else if served.surface != clean.surface {
            "s1"
        } else if served.page != clean.page {
            "s2"
        } else {
            continue;
        };
        let _ = write!(out, "{}[{}] {field}", clean.kind.tag(), clean.ordinal);
        return out;
    }
    out
}
