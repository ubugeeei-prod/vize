//! Selected-file semantics on the same snapshot, without whole-program work.

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
