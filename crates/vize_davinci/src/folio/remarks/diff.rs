//! Remarks-diff (TS-32): what changed between two corpus remark sets —
//! LLVM's `opt-diff`, keyed rather than line-diffed.
//!
//! A remark's identity is `(path, stage, pass, name, span)` plus its
//! occurrence index among remarks sharing that identity; its *state* is
//! `(kind, args)`. Matching identities whose state differs are classified;
//! unmatched ones are added or removed. **`Regressed` (`applied → missed`)
//! is the class TS-32 gates on**: an optimization the compiler used to make
//! and no longer does, invisible to an output diff when the output merely
//! got worse rather than wrong.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use vize_s0::String;

use super::corpus::CorpusRemark;
use crate::pass::observer::{RecordedRemark, RemarkKind};

/// How one remark identity changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeKind {
    /// `applied → missed`: a lost optimization.
    Regressed,
    /// `missed → applied`: a gained optimization.
    Improved,
    /// Any other kind change (to or from `analysis`).
    Rekinded,
    /// Same kind, different arguments (a blocker moved).
    Reargued,
    /// Present only after.
    Added,
    /// Present only before.
    Removed,
}

impl ChangeKind {
    /// Stable spelling for reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Regressed => "regressed",
            Self::Improved => "improved",
            Self::Rekinded => "rekinded",
            Self::Reargued => "reargued",
            Self::Added => "added",
            Self::Removed => "removed",
        }
    }
}

/// One changed remark identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemarkChange {
    /// The class of change.
    pub kind: ChangeKind,
    /// The file.
    pub path: String,
    /// The remark before, if it existed.
    pub before: Option<RecordedRemark>,
    /// The remark after, if it exists.
    pub after: Option<RecordedRemark>,
}

type Identity<'a> = (&'a str, &'a str, &'a str, &'a str, u32, u32, usize);

fn identities(remarks: &[CorpusRemark]) -> BTreeMap<Identity<'_>, &CorpusRemark> {
    let mut seen: BTreeMap<(&str, &str, &str, &str, u32, u32), usize> = BTreeMap::new();
    let mut out = BTreeMap::new();
    for entry in remarks {
        let remark = &entry.remark;
        let base = (
            entry.path.as_str(),
            remark.stage.as_str(),
            remark.pass.as_str(),
            remark.name.as_str(),
            remark.span.start,
            remark.span.end,
        );
        let occurrence = seen.entry(base).or_insert(0);
        out.insert(
            (base.0, base.1, base.2, base.3, base.4, base.5, *occurrence),
            entry,
        );
        *occurrence += 1;
    }
    out
}

fn classify(before: &RecordedRemark, after: &RecordedRemark) -> Option<ChangeKind> {
    match (before.kind, after.kind) {
        (RemarkKind::Applied, RemarkKind::Missed) => Some(ChangeKind::Regressed),
        (RemarkKind::Missed, RemarkKind::Applied) => Some(ChangeKind::Improved),
        (from, to) if from != to => Some(ChangeKind::Rekinded),
        _ if before.args != after.args => Some(ChangeKind::Reargued),
        _ => None,
    }
}

/// Every changed identity between `before` and `after`: changed and
/// removed identities in identity order, then added ones in identity order.
#[must_use]
pub fn diff_corpus(before: &[CorpusRemark], after: &[CorpusRemark]) -> Vec<RemarkChange> {
    let old = identities(before);
    let new = identities(after);
    let mut changes = Vec::new();
    for (identity, previous) in &old {
        match new.get(identity) {
            Some(current) => {
                if let Some(kind) = classify(&previous.remark, &current.remark) {
                    changes.push(RemarkChange {
                        kind,
                        path: previous.path.clone(),
                        before: Some(previous.remark.clone()),
                        after: Some(current.remark.clone()),
                    });
                }
            }
            None => changes.push(RemarkChange {
                kind: ChangeKind::Removed,
                path: previous.path.clone(),
                before: Some(previous.remark.clone()),
                after: None,
            }),
        }
    }
    for (identity, current) in &new {
        if !old.contains_key(identity) {
            changes.push(RemarkChange {
                kind: ChangeKind::Added,
                path: current.path.clone(),
                before: None,
                after: Some(current.remark.clone()),
            });
        }
    }
    changes
}
