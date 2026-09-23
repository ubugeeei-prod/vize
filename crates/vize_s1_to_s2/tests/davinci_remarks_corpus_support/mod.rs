//! C-13's first version: the missed-remarks backlog, rendered from the
//! TS-32 corpus by `vize_davinci::folio::remarks::backlog::mine_missed` and
//! pinned as `docs/davinci/plan/remarks-backlog.md`.

use vize_davinci::folio::remarks::backlog::mine_missed;
use vize_davinci::folio::remarks::corpus::RemarkCorpus;
use vize_davinci::pass::RemarkKind;
use vize_s0::{String, cstr};

/// The committed backlog document, relative to the repo root.
pub const BACKLOG_REL: &str = "docs/davinci/plan/remarks-backlog.md";

/// Render the backlog for `corpus` (formatter-stable Markdown).
pub fn backlog_markdown(corpus: &RemarkCorpus) -> String {
    let count = |kind| {
        corpus
            .entries
            .iter()
            .filter(|entry| entry.remark.kind == kind)
            .count()
    };
    let items = mine_missed(&corpus.entries);
    let mut out = String::from(
        "# Missed-remarks backlog (C-13)\n\n\
         > [!NOTE]\n\
         > Generated; do not edit. Mined from the TS-32 corpus baseline\n\
         > (`tests/_fixtures/davinci-remarks-baseline.folio`) by\n\
         > `crates/vize_s1_to_s2/tests/davinci_remarks_corpus.rs`, which pins this file\n\
         > and rewrites it under `UPDATE_REMARKS_BASELINE=1`. Each item is one missed\n\
         > reason (a remark's arguments after its subject, per\n\
         > [remarks-format.md](./remarks-format.md)), ranked by corpus hits: the\n\
         > optimization backlog the P3-13 remarks mine for continuous task C-13.\n\n",
    );
    out.push_str(
        cstr!(
            "Corpus: {} files, {} remarks ({} applied, {} missed), {} missed reasons.\n\n",
            corpus.files.len(),
            corpus.entries.len(),
            count(RemarkKind::Applied),
            count(RemarkKind::Missed),
            items.len()
        )
        .as_str(),
    );
    for (index, item) in items.iter().enumerate() {
        let reason = if item.reason.is_empty() {
            String::from("(subject only)")
        } else {
            cstr!("`{}`", item.reason)
        };
        out.push_str(
            cstr!(
                "{}. `{}.{} {}` {} - {} hits in {} files; first at `{}` @{}:{}\n",
                index + 1,
                item.stage,
                item.pass,
                item.name,
                reason,
                item.hits,
                item.files,
                item.example.0,
                item.example.1.start,
                item.example.1.end
            )
            .as_str(),
        );
    }
    out
}
