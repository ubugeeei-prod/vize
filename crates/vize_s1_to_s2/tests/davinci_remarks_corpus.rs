//! TS-32 — the corpus remarks-diff (P3-13).
//!
//! Every `.vue` file under `tests/_fixtures` (the pinned `_git` submodules
//! and `node_modules` excluded, so the sweep is the same on every checkout)
//! has its template lowered and run through the S2 transform pipeline under
//! a remark collector. The resulting `[remarks-corpus]` page — swept files,
//! every remark keyed by file with file-absolute spans — must equal the
//! committed baseline `tests/_fixtures/davinci-remarks-baseline.folio`
//! **exactly**. A difference fails with the remarks-diff classified
//! (`regressed` = `applied → missed`, `improved`, `rekinded`, `reargued`,
//! `added`, `removed`), so an optimization lost without any output diff is
//! still a red build.
//!
//! Re-bless deliberately with `UPDATE_REMARKS_BASELINE=1`. The bless itself
//! refuses any **unexplained** regression: each `applied → missed`
//! transition must first be listed in the baseline's
//! `[remarks-corpus.explained]` section with its `reason` — the ledger that
//! makes "no unexplained applied → missed transitions" (TS-32's oracle)
//! machine-checked. A plain run also rejects an explanation that no longer
//! names a missed remark, so the ledger cannot rot.
//!
//! Run:
//!
//! ```text
//! cargo test -p vize_s1_to_s2 --features davinci-differential \
//!     --test davinci_remarks_corpus -- --nocapture
//! ```

mod davinci_remarks_corpus_support;

use davinci_remarks_corpus_support as support;

use std::path::{Path, PathBuf};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_davinci::folio::remarks::corpus::{CorpusRemark, RemarkCorpus};
use vize_davinci::folio::remarks::diff::{ChangeKind, RemarkChange, diff_corpus};
use vize_davinci::folio::remarks::entry_line;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::{RemarkCollector, RemarkKind};
use vize_s0::{Allocator, Span, String, cstr};
use vize_s1::parse;
use vize_s1_to_s2::pass::run_transform;
use vize_s1_to_s2::{LegacyCaps, lower_with_caps};

const CORPUS_REL: &str = "tests/_fixtures";
const BASELINE_REL: &str = "tests/_fixtures/davinci-remarks-baseline.folio";
const UPDATE_ENV: &str = "UPDATE_REMARKS_BASELINE";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every swept `.vue` path, repo-relative, `/`-separated, sorted.
fn corpus_files(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .expect("corpus directory reads")
            .map(|entry| entry.expect("readable corpus entry").path())
            .collect();
        entries.sort();
        for path in entries {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if path.is_dir() {
                if name != "_git" && name != "node_modules" {
                    walk(&path, root, out);
                }
            } else if name.ends_with(".vue") {
                let relative = path.strip_prefix(root).expect("path under the repo root");
                let parts: Vec<&str> = relative
                    .components()
                    .map(|part| part.as_os_str().to_str().expect("UTF-8 corpus path"))
                    .collect();
                out.push(String::from(parts.join("/").as_str()));
            }
        }
    }
    let mut files = Vec::new();
    walk(&root.join(CORPUS_REL), root, &mut files);
    files.sort();
    files
}

/// The file's template remarks, spans shifted to file offsets. Files
/// without an inline HTML template contribute none.
fn file_remarks(path: &str, source: &str, out: &mut Vec<CorpusRemark>) {
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return;
    };
    let Some(template) = descriptor.template.as_ref() else {
        return;
    };
    let html = template.lang.as_deref().is_none_or(|lang| lang == "html");
    if template.src.is_some() || !html {
        return;
    }
    let offset = u32::try_from(template.loc.start).expect("fixture offsets fit u32");
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, &template.content);
    let mut lowered = lower_with_caps(&allocator, &tree, &errors, LegacyCaps::VUE3);
    let mut collector = RemarkCollector::new();
    let _facts = run_transform(&mut lowered, &mut collector);
    for mut remark in collector.finish() {
        remark.span = Span::new(remark.span.start + offset, remark.span.end + offset);
        out.push(CorpusRemark {
            path: String::from(path),
            remark,
        });
    }
}

fn current_corpus(root: &Path) -> RemarkCorpus {
    let files = corpus_files(root);
    let mut entries = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(root.join(path.as_str())).expect("fixture reads");
        file_remarks(path, &source, &mut entries);
    }
    RemarkCorpus {
        files,
        entries,
        explained: Vec::new(),
    }
}

fn describe(change: &RemarkChange) -> String {
    let side = |remark: Option<&vize_davinci::pass::observer::RecordedRemark>| {
        remark.map_or_else(|| String::from("-"), entry_line)
    };
    cstr!(
        "{} {} | {} => {}",
        change.kind.as_str(),
        change.path,
        side(change.before.as_ref()),
        side(change.after.as_ref())
    )
}

fn report(changes: &[RemarkChange]) -> String {
    let mut out = String::default();
    for kind in [
        ChangeKind::Regressed,
        ChangeKind::Improved,
        ChangeKind::Rekinded,
        ChangeKind::Reargued,
        ChangeKind::Added,
        ChangeKind::Removed,
    ] {
        let of_kind: Vec<&RemarkChange> = changes.iter().filter(|c| c.kind == kind).collect();
        if of_kind.is_empty() {
            continue;
        }
        out.push_str(cstr!("{}: {}\n", kind.as_str(), of_kind.len()).as_str());
        for change in of_kind.iter().take(20) {
            out.push_str(cstr!("  {}\n", describe(change)).as_str());
        }
    }
    out
}

fn unexplained_regressions<'c>(
    changes: &'c [RemarkChange],
    committed: &RemarkCorpus,
) -> Vec<&'c RemarkChange> {
    changes
        .iter()
        .filter(|change| change.kind == ChangeKind::Regressed)
        .filter(|change| {
            let after = change
                .after
                .as_ref()
                .expect("a regression has an after side");
            !committed
                .explained
                .iter()
                .any(|explained| explained.names(change.path.as_str(), after))
        })
        .collect()
}

#[test]
fn corpus_remarks_match_the_committed_baseline() {
    let root = repo_root();
    let mut current = current_corpus(&root);
    let baseline_path = root.join(BASELINE_REL);
    let committed_text = std::fs::read_to_string(&baseline_path).unwrap_or_default();
    let committed = if committed_text.is_empty() {
        RemarkCorpus::default()
    } else {
        RemarkCorpus::parse(&committed_text).expect("the committed baseline parses")
    };
    let changes = diff_corpus(&committed.entries, &current.entries);
    let applied = current
        .entries
        .iter()
        .filter(|e| e.remark.kind == RemarkKind::Applied)
        .count();
    let missed = current
        .entries
        .iter()
        .filter(|e| e.remark.kind == RemarkKind::Missed)
        .count();
    eprintln!(
        "ts-32 remarks corpus: files={} remarks={} applied={applied} missed={missed} changes={}",
        current.files.len(),
        current.entries.len(),
        changes.len()
    );

    if std::env::var_os(UPDATE_ENV).is_some() {
        let unexplained = unexplained_regressions(&changes, &committed);
        assert!(
            unexplained.is_empty(),
            "refusing to bless {} unexplained applied -> missed transition(s); list each in \
             [remarks-corpus.explained] with a reason first:\n{}",
            unexplained.len(),
            unexplained
                .iter()
                .map(|c| describe(c))
                .collect::<Vec<_>>()
                .join("\n")
        );
        current.explained = committed.explained;
        std::fs::write(
            &baseline_path,
            current.print_to_string(FolioMode::Full).as_bytes(),
        )
        .expect("baseline writes");
        std::fs::write(
            root.join(support::BACKLOG_REL),
            support::backlog_markdown(&current).as_bytes(),
        )
        .expect("backlog writes");
        return;
    }

    // The ledger cannot rot: every explanation names a baseline `missed`
    // remark.
    for explained in &committed.explained {
        let resolves = committed.entries.iter().any(|entry| {
            entry.remark.kind == RemarkKind::Missed
                && explained.names(entry.path.as_str(), &entry.remark)
        });
        assert!(
            resolves,
            "explained regression {} {}.{} {} @{}:{} names no missed baseline remark",
            explained.path,
            explained.stage,
            explained.pass,
            explained.name,
            explained.span.start,
            explained.span.end
        );
    }
    assert_eq!(current.files, committed.files, "the swept file set moved");
    assert!(
        changes.is_empty(),
        "TS-32 remarks-diff is not clean ({} change(s)); if intended, re-bless with \
         {UPDATE_ENV}=1 (regressions need an explanation):\n{}",
        changes.len(),
        report(&changes)
    );
    current.explained = committed.explained;
    assert_eq!(
        current.print_to_string(FolioMode::Full).as_str(),
        committed_text.as_str(),
        "the baseline is not in canonical form; re-bless with {UPDATE_ENV}=1"
    );
    // C-13: the committed backlog is exactly the one mined from the corpus.
    let backlog = std::fs::read_to_string(root.join(support::BACKLOG_REL)).unwrap_or_default();
    assert_eq!(
        support::backlog_markdown(&current).as_str(),
        backlog.as_str(),
        "{} is stale; regenerate with {UPDATE_ENV}=1",
        support::BACKLOG_REL
    );
}
