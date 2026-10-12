//! Exact generated integration policy and its pinned body fields.

pub(super) fn generated(source_number: u64, cut: &str, source_head: &str, tag: &str) -> String {
    format!(
        "Integrate only generated {} version metadata into moving main. Source PR #{} remains an immutable draft receipt at H={}. The maintainer owns protected queue admission; this command never auto-merges this PR or writes main. Admit this integration only after immutable H/R qualification has succeeded: all required H checks, all five full H gates, every release build, and full preflight must be terminal green. Actual delivery is checked against its final first parent, not the initial base.\n\n<!-- vize-release-pin-source: {} -->\n<!-- vize-release-pin-cut: {} -->\n<!-- vize-release-pin-head: {} -->\n<!-- vize-release-pin-tag: {} -->\n",
        tag,
        source_number,
        source_head,
        source_number,
        cut,
        source_head,
        tag
    )
}
