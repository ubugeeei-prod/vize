//! The per-file metamorphic driver and the run-level scope proof.
//!
//! Every input file is parsed and lowered once; every enumerated site
//! then gets its own fresh parse, one mutation, and one oracle
//! comparison against the original's normalized folio. The run counts
//! sites, applied mutations, predicate skips and beyond-cap sites per
//! mutator, and [`Report::verdict`] fails a run whose total mutation
//! count is zero — a suite that silently degenerates to no mutations
//! must fail loudly, the TS-11 scope-proof discipline.

use std::path::{Path, PathBuf};

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, String, append, cstr};
use vize_s1::parse;
use vize_s1_to_s2::lower;
use vize_s2::folio::DisegnoFolio;

use super::mutators::{Mutant, apply, merge_text};
use super::normalize::{Normalization, normalized_display};
use super::sites::{Detail, Kind, Site, enumerate};

/// Applied mutations per mutator per file are capped for corpus-scale
/// runs; capped-out sites are counted separately from skips — they were
/// safe, just not attempted.
pub const PER_MUTATOR_FILE_CAP: u64 = 8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    pub sites: u64,
    pub applied: u64,
    pub skipped: u64,
    pub beyond_cap: u64,
}

#[derive(Debug, Default)]
pub struct Report {
    pub files: u64,
    pub unreadable: u64,
    pub reorder: Tally,
    pub wrap: Tally,
    pub split: Tally,
    pub merge: Tally,
    pub whitespace: Tally,
    pub failures: Vec<String>,
}

impl Report {
    fn tally(&mut self, kind: Kind) -> &mut Tally {
        match kind {
            Kind::Reorder => &mut self.reorder,
            Kind::Wrap => &mut self.wrap,
            Kind::Split => &mut self.split,
            Kind::Merge => &mut self.merge,
            Kind::Whitespace => &mut self.whitespace,
        }
    }

    pub fn mutations(&self) -> u64 {
        [
            self.reorder,
            self.wrap,
            self.split,
            self.merge,
            self.whitespace,
        ]
        .iter()
        .map(|tally| tally.applied)
        .sum()
    }

    /// The scope-proof line the lanes print and the tooling gate parses.
    pub fn scope_line(&self, label: &str) -> String {
        let mut line = cstr!(
            "metamorphic {label}: files={} unreadable={} mutations={}",
            self.files,
            self.unreadable,
            self.mutations()
        );
        for (name, tally) in [
            ("reorder", self.reorder),
            ("wrap", self.wrap),
            ("split", self.split),
            ("merge", self.merge),
            ("whitespace", self.whitespace),
        ] {
            append!(
                line,
                " {name}={}/{}sites (skips={} beyond-cap={})",
                tally.applied,
                tally.sites,
                tally.skipped,
                tally.beyond_cap
            );
        }
        line
    }

    /// The run-level scope proof: a zero-mutation run fails.
    pub fn verdict(&self, label: &str) -> Result<(), String> {
        if self.mutations() == 0 {
            return Err(cstr!(
                "metamorphic {label}: zero mutations were applied — the run proves nothing \
                 ({} files, {} sites, {} skips); a degenerated suite must fail, not pass",
                self.files,
                self.total_sites(),
                self.total_skips()
            ));
        }
        Ok(())
    }

    fn total_sites(&self) -> u64 {
        [
            self.reorder,
            self.wrap,
            self.split,
            self.merge,
            self.whitespace,
        ]
        .iter()
        .map(|tally| tally.sites)
        .sum()
    }

    fn total_skips(&self) -> u64 {
        [
            self.reorder,
            self.wrap,
            self.split,
            self.merge,
            self.whitespace,
        ]
        .iter()
        .map(|tally| tally.skipped)
        .sum()
    }
}

/// The normalization each mutator's justification declares
/// (`normalize.rs` for the rules, `mutators.rs` for the arguments).
/// Split/merge and whitespace declare **none** since the P2-9 ratchet —
/// the lowering's own condense-and-merge is the canonical operation.
fn rules_for(kind: Kind) -> Normalization {
    match kind {
        Kind::Reorder => Normalization::REORDER,
        Kind::Wrap | Kind::Split | Kind::Merge | Kind::Whitespace => Normalization::NONE,
    }
}

fn folio_of_source(source: &str) -> DisegnoFolio {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let lowered = lower(&allocator, &tree, &errors);
    DisegnoFolio::of(&lowered.root.ops)
}

/// Run every mutator over one source, accumulating into `report`.
pub fn run_source(source: &str, context: &str, report: &mut Report) {
    report.files += 1;
    let original = folio_of_source(source);
    let sites = {
        let allocator = Allocator::new();
        let (tree, _errors) = parse(&allocator, source);
        enumerate(&tree)
    };
    let mut applied_per_kind = [0u64; 5];
    for site in &sites {
        let slot = site.kind as usize;
        let tally = report.tally(site.kind);
        tally.sites += 1;
        if site.skip.is_some() {
            tally.skipped += 1;
            continue;
        }
        if applied_per_kind[slot] >= PER_MUTATOR_FILE_CAP {
            tally.beyond_cap += 1;
            continue;
        }
        applied_per_kind[slot] += 1;
        tally.applied += 1;
        check_site(source, context, site, &original, report);
    }
}

fn check_site(
    source: &str,
    context: &str,
    site: &Site,
    original: &DisegnoFolio,
    report: &mut Report,
) {
    let rules = rules_for(site.kind);
    let allocator = Allocator::new();
    let (mut tree, errors) = parse(&allocator, source);
    let mutant_folio = match apply(&allocator, &mut tree, site) {
        Mutant::Source(mutant_source) => folio_of_source(mutant_source.as_str()),
        Mutant::InPlace => {
            let lowered = lower(&allocator, &tree, &errors);
            DisegnoFolio::of(&lowered.root.ops)
        }
    };
    let expected = normalized_display(original, rules);
    let actual = normalized_display(&mutant_folio, rules);
    if expected != actual {
        report.failures.push(divergence(
            context,
            site,
            "normalized folios diverge",
            &expected,
            &actual,
        ));
    }
    if let Detail::SplitAt { .. } = site.detail {
        // The exact inverse: re-merging the split reproduces the original
        // `Full`-mode folio byte-for-byte (no normalization at all).
        merge_text(&mut tree, &site.path);
        let lowered = lower(&allocator, &tree, &errors);
        let remerged = DisegnoFolio::of(&lowered.root.ops);
        let expected_full = original.print_to_string(FolioMode::Full);
        let actual_full = remerged.print_to_string(FolioMode::Full);
        if expected_full != actual_full {
            report.failures.push(divergence(
                context,
                site,
                "split-then-merge is not the identity",
                &expected_full,
                &actual_full,
            ));
        }
    }
}

/// One failure entry: the site, the first divergent line, and both full
/// artifacts when they are small enough to read.
fn divergence(context: &str, site: &Site, what: &str, expected: &str, actual: &str) -> String {
    let first = expected
        .lines()
        .zip(actual.lines())
        .position(|(a, b)| a != b)
        .map_or_else(
            || String::from("line counts differ"),
            |index| {
                cstr!(
                    "first divergence at line {}:\n  original: {}\n  mutant:   {}",
                    index + 1,
                    expected.lines().nth(index).unwrap_or(""),
                    actual.lines().nth(index).unwrap_or("")
                )
            },
        );
    let bodies = if expected.len() + actual.len() <= 4096 {
        cstr!("\n--- original ---\n{expected}\n--- mutant ---\n{actual}")
    } else {
        String::default()
    };
    cstr!(
        "{context}: {what} ({:?} at path {:?}, {:?})\n{first}{bodies}",
        site.kind,
        site.path,
        site.detail
    )
}

/// Every `.vue` file under `root`, sorted, `node_modules` pruned — the
/// P2-8 corpus-walk shape.
pub fn collect_vue_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            if child.file_name().is_some_and(|name| name == "node_modules") {
                continue;
            }
            collect_vue_files(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

/// Run the suite over a file list.
pub fn run_files(files: &[PathBuf]) -> Report {
    let mut report = Report::default();
    for file in files {
        match std::fs::read_to_string(file) {
            Ok(source) => run_source(&source, &file.to_string_lossy(), &mut report),
            Err(_) => report.unreadable += 1,
        }
    }
    report
}
