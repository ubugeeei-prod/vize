//! TS-32 for P3-10: the corpus remarks-diff over S3 extraction.
//!
//! The same sweep as the S2 remarks corpus (every `.vue` file under
//! `tests/_fixtures`, `_git` and `node_modules` excluded): each inline HTML
//! template is lowered S1 -> S2 -> S3 and run through the `s3` `OPTIMIZE`
//! pipeline at `-O3` under a remark collector. The `[remarks-corpus]` page
//! must equal `tests/_fixtures/davinci-s3-remarks-baseline.folio` exactly.
//!
//! Re-bless with `UPDATE_REMARKS_BASELINE=1`. The bless refuses every
//! `applied -> missed` transition not listed with a reason in
//! `[remarks-corpus.explained]`, and a plain run rejects an explanation that
//! names no missed baseline remark. Every swept plan must also verify and keep
//! the partition export current, so the gate doubles as a corpus-wide TS-11
//! precondition check for the overlay.

use std::path::{Path, PathBuf};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_davinci::folio::remarks::corpus::{CorpusRemark, RemarkCorpus};
use vize_davinci::folio::remarks::diff::{ChangeKind, RemarkChange, diff_corpus};
use vize_davinci::folio::remarks::entry_line;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::{RemarkCollector, RemarkKind};
use vize_s0::{Allocator, Span, String, cstr};
use vize_s3::extract::OptTier;
use vize_s3::optimize::optimize;
use vize_s3::verify::verify;

const CORPUS_REL: &str = "tests/_fixtures";
const BASELINE_REL: &str = "tests/_fixtures/davinci-s3-remarks-baseline.folio";
const UPDATE_ENV: &str = "UPDATE_REMARKS_BASELINE";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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
                if name != "_git" && name != "_git-worktrees" && name != "node_modules" {
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

/// The file's extraction remarks, spans shifted to file offsets.
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
    let (tree, errors) = vize_s1::parse(&allocator, &template.content);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let mut lowered = vize_s2_to_s3::lower(&allocator, &s2.root);
    let mut collector = RemarkCollector::new();
    optimize(&mut lowered.program, OptTier::O3, &mut collector).expect("closed pipeline");
    assert_eq!(verify(&lowered.program), [], "{path}");
    assert_eq!(lowered.partition.stale(&lowered.program), None, "{path}");
    for mut remark in collector.finish() {
        remark.span = Span::new(remark.span.start + offset, remark.span.end + offset);
        out.push(CorpusRemark {
            path: String::from(path),
            remark,
        });
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
fn s3_extraction_remarks_match_the_committed_baseline() {
    let root = repo_root();
    let files = corpus_files(&root);
    let mut entries = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(root.join(path.as_str())).expect("fixture reads");
        file_remarks(path, &source, &mut entries);
    }
    let mut current = RemarkCorpus {
        files,
        entries,
        explained: Vec::new(),
    };
    let baseline_path = root.join(BASELINE_REL);
    let committed_text = std::fs::read_to_string(&baseline_path).unwrap_or_default();
    let committed = if committed_text.is_empty() {
        RemarkCorpus::default()
    } else {
        RemarkCorpus::parse(&committed_text).expect("the committed baseline parses")
    };
    let changes = diff_corpus(&committed.entries, &current.entries);
    let count = |kind| {
        current
            .entries
            .iter()
            .filter(|e| e.remark.kind == kind)
            .count()
    };
    eprintln!(
        "ts-32 s3 remarks corpus: files={} remarks={} applied={} missed={} changes={}",
        current.files.len(),
        current.entries.len(),
        count(RemarkKind::Applied),
        count(RemarkKind::Missed),
        changes.len()
    );

    if std::env::var_os(UPDATE_ENV).is_some() {
        let unexplained = unexplained_regressions(&changes, &committed);
        assert!(
            unexplained.is_empty(),
            "refusing to bless {} unexplained applied -> missed transition(s):\n{}",
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
        return;
    }

    for explained in &committed.explained {
        let resolves = committed.entries.iter().any(|entry| {
            entry.remark.kind == RemarkKind::Missed
                && explained.names(entry.path.as_str(), &entry.remark)
        });
        assert!(
            resolves,
            "explained regression {} {} @{}:{} names no missed baseline remark",
            explained.path, explained.name, explained.span.start, explained.span.end
        );
    }
    assert_eq!(current.files, committed.files, "the swept file set moved");
    assert!(
        changes.is_empty(),
        "TS-32 s3 remarks-diff is not clean ({} change(s)); if intended, re-bless with \
         {UPDATE_ENV}=1:\n{}",
        changes.len(),
        changes
            .iter()
            .take(40)
            .map(describe)
            .collect::<Vec<_>>()
            .join("\n")
    );
    current.explained = committed.explained;
    assert_eq!(
        current.print_to_string(FolioMode::Full).as_str(),
        committed_text.as_str(),
        "the baseline is not in canonical form; re-bless with {UPDATE_ENV}=1"
    );
}
