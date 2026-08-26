//! Exact-edit assertions shared by the real-Corsa session scenarios.
//!
//! Split out of [`super`] to keep that file inside the per-file line budget;
//! `super` re-imports these so the sibling scenario modules keep referring to
//! them through `super::`.

use tower_lsp::lsp_types::{PrepareRenameResponse, Range, Url, WorkspaceEdit};
use vize_canon::CorsaBridge;
use vize_s0::cstr;

use super::super::super::canonical;
use crate::ide::IdeContext;

pub(super) fn strict_rename(
    ctx: &IdeContext<'_>,
    bridge: &CorsaBridge,
    new_name: &str,
) -> Result<WorkspaceEdit, String> {
    let answer = crate::runtime::block_on(canonical::rename_strict(ctx, new_name, Some(bridge)))
        .map_err(|error| cstr!("strict rename failed: {error}"))?;
    let canonical::Answer::Available(Some(edit)) = answer else {
        return Err("strict rename returned no canonical edit".to_owned());
    };
    Ok(edit)
}

pub(super) fn assert_exact_event_edit(
    edit: &WorkspaceEdit,
    parent: (&Url, &str, &str, &str),
    child: (&Url, &str, &str, &str),
) -> Result<(), String> {
    let changes = edit
        .changes
        .as_ref()
        .ok_or_else(|| "rename did not return plain workspace changes".to_owned())?;
    if changes.len() != 2 || !changes.contains_key(parent.0) || !changes.contains_key(child.0) {
        return Err(cstr!("unexpected rename URI set: {changes:#?}").into());
    }
    for (uri, source, old_name, new_name) in [parent, child] {
        let edits = changes
            .get(uri)
            .ok_or_else(|| cstr!("missing edit for {uri}"))?;
        if edits.len() != 1
            || authored_text(source, edits[0].range) != old_name
            || edits[0].new_text != new_name
        {
            return Err(
                cstr!("expected one exact {old_name} -> {new_name} edit: {changes:#?}").into(),
            );
        }
    }
    Ok(())
}

pub(super) fn prepare_range(response: &PrepareRenameResponse) -> Range {
    match response {
        PrepareRenameResponse::Range(range)
        | PrepareRenameResponse::RangeWithPlaceholder { range, .. } => *range,
        PrepareRenameResponse::DefaultBehavior { .. } => panic!("expected an authored range"),
    }
}

pub(super) fn authored_text(source: &str, range: Range) -> &str {
    let start = crate::ide::position_to_offset(source, range.start.line, range.start.character)
        .expect("valid source start");
    let end = crate::ide::position_to_offset(source, range.end.line, range.end.character)
        .expect("valid source end");
    assert!(start <= end, "inverted authored range");
    &source[start..end]
}
