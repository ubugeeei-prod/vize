//! Same-snapshot requested names and whole-category main-source admission.

use super::conversion::NativeDiagnostic;
use vize_l0::{FxHashSet, String};

pub(super) fn names<'a>(
    uris: &[String],
    mut source_name: impl FnMut(&str) -> Option<&'a str>,
) -> Option<Vec<&'a str>> {
    let mut seen = FxHashSet::default();
    let mut selected = Vec::new();
    for uri in uris {
        let name = source_name(uri)?;
        if seen.insert(name) {
            selected.push(name);
        }
    }
    Some(selected)
}

pub(super) fn decode(
    value: serde_json::Value,
    selected_file: Option<&str>,
) -> Option<Vec<NativeDiagnostic>> {
    let diagnostics = serde_json::from_value::<Option<Vec<NativeDiagnostic>>>(value)
        .ok()?
        .unwrap_or_default();
    // A foreign main diagnostic cannot inherit this requested document's
    // ownership. Related locations retain the original snapshot-text provider.
    if selected_file.is_some_and(|file| !diagnostics.iter().all(|row| row.belongs_to(file))) {
        return None;
    }
    Some(diagnostics)
}

pub(super) fn decode_project(
    value: serde_json::Value,
    members: &FxHashSet<&str>,
    requested: &FxHashSet<&str>,
) -> Option<Vec<NativeDiagnostic>> {
    // Decode the entire response before omitting anything. A malformed or
    // foreign unrequested row must refuse this whole category as well.
    let diagnostics = decode(value, None)?;
    if !diagnostics
        .iter()
        .all(|row| !row.file_name().is_empty() && members.contains(row.file_name()))
    {
        return None;
    }
    // Preserve native category order. Only retained requested rows need the
    // existing position/related-text conversion; no later disk read is added.
    Some(
        diagnostics
            .into_iter()
            .filter(|row| requested.contains(row.file_name()))
            .collect(),
    )
}
