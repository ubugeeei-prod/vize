//! Same-snapshot requested names and whole-category main-source admission.

use super::conversion::NativeDiagnostic;
use serde::Deserialize;
use vize_l0::{FxHashSet, String};

mod raw_schema;

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

#[cfg(test)]
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
    let rows = match value {
        serde_json::Value::Null => return Some(Vec::new()),
        serde_json::Value::Array(rows) => rows,
        _ => return None,
    };
    let mut retained = Vec::new();
    for row in rows {
        let name = raw_schema::main_name(&row)?;
        if name.is_empty() || !members.contains(name) {
            return None;
        }
        if requested.contains(name) {
            // Decode retained rows once with the original owned schema.
            retained.push(serde_json::from_value::<NativeDiagnostic>(row).ok()?);
        } else {
            // Validate every omitted field, including recursive children,
            // without constructing owned diagnostic strings or child vectors.
            raw_schema::Diagnostic::deserialize(&row).ok()?;
        }
    }
    // Any late malformed/foreign row discards the entire local category.
    // Native order and the original requested/related converter stay intact.
    Some(retained)
}
