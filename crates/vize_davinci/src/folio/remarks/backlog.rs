//! Missed-remarks mining (C-13): the corpus's `missed` remarks grouped by
//! *reason*, ranked by how often the corpus hits them — the optimization
//! backlog, mined rather than guessed.
//!
//! A remark's first argument is its **subject** (what the decision is
//! about: `tag`, `component`); the remaining arguments are its **reason**
//! (why it missed: `blocker`, `op`, `rule`). That split is a vocabulary rule
//! (`remarks-format.md`), and it is what makes grouping pass-agnostic: two
//! missed `static-subtree` remarks on different tags blocked by the same
//! `ui.on` binding are one backlog item with two hits.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use vize_s0::{Span, String};

use super::args_text;
use super::corpus::CorpusRemark;
use crate::pass::observer::RemarkKind;

/// One backlog item: a missed-remark reason and its corpus footprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BacklogItem {
    /// Emitting stage.
    pub stage: String,
    /// Emitting pass.
    pub pass: String,
    /// Remark name.
    pub name: String,
    /// The reason arguments in entry-line spelling (empty when the remark
    /// carries only a subject).
    pub reason: String,
    /// Missed remarks with this reason.
    pub hits: usize,
    /// Distinct files they occur in.
    pub files: usize,
    /// The first occurrence in corpus order: path and span.
    pub example: (String, Span),
}

/// Group every `missed` remark by `(stage, pass, name, reason)`, ranked by
/// hits descending, then by the group key ascending (a total order, so the
/// backlog is deterministic).
#[must_use]
pub fn mine_missed(entries: &[CorpusRemark]) -> Vec<BacklogItem> {
    type Key = (String, String, String, String);
    /// Hits, distinct files, first occurrence.
    type Group<'e> = (usize, BTreeSet<&'e str>, (String, Span));
    let mut groups: BTreeMap<Key, Group<'_>> = BTreeMap::new();
    for entry in entries {
        let remark = &entry.remark;
        if remark.kind != RemarkKind::Missed {
            continue;
        }
        let reason = args_text(remark.args.get(1..).unwrap_or(&[]));
        let key = (
            remark.stage.clone(),
            remark.pass.clone(),
            remark.name.clone(),
            reason,
        );
        let group = groups
            .entry(key)
            .or_insert_with(|| (0, BTreeSet::new(), (entry.path.clone(), remark.span)));
        group.0 += 1;
        group.1.insert(entry.path.as_str());
    }
    let mut items: Vec<BacklogItem> = groups
        .into_iter()
        .map(
            |((stage, pass, name, reason), (hits, files, example))| BacklogItem {
                stage,
                pass,
                name,
                reason,
                hits,
                files: files.len(),
                example,
            },
        )
        .collect();
    // Stable: equal hit counts keep the key order the map produced.
    items.sort_by_key(|item| core::cmp::Reverse(item.hits));
    items
}
