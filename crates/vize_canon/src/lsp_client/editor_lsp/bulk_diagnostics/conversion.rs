//! Native categories projected using the pinned LSP converter's semantics.

use super::positions::Positions;
use crate::file_uri::path_to_file_uri;
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Uri,
};
use serde::Deserialize;
use std::{path::Path, str::FromStr};
#[cfg(test)]
use vize_l0::FxHashSet;
use vize_l0::{FxHashMap, String};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NativeDiagnostic {
    #[serde(default)]
    file_name: String,
    pos: u32,
    end: u32,
    code: i32,
    category: u8,
    text: String,
    #[serde(default)]
    message_chain: Vec<NativeDiagnostic>,
    #[serde(default)]
    related_information: Vec<NativeDiagnostic>,
}

#[cfg(test)]
pub(super) fn requested_members_are_present(uris: &[String], names: &[String]) -> bool {
    // Path equality normalizes interior `.` and repeated separators, whereas
    // URI keys retain their lexical bytes. Compare the actual grouping keys.
    let members = names
        .iter()
        .map(|name| path_to_file_uri(Path::new(name.as_str())))
        .collect::<FxHashSet<_>>();
    uris.iter().all(|uri| members.contains(uri.as_str()))
}

#[cfg(test)]
pub(super) fn project_diagnostics(
    categories: &[Vec<NativeDiagnostic>],
    uris: &[String],
    documents: &FxHashMap<String, String>,
) -> Option<Vec<Vec<Diagnostic>>> {
    match project_diagnostics_with_source(categories, uris, documents, |_| {
        Ok::<_, std::convert::Infallible>(None)
    }) {
        Ok(value) => value,
        Err(error) => match error {},
    }
}

pub(super) fn project_diagnostics_with_source<E>(
    categories: &[Vec<NativeDiagnostic>],
    uris: &[String],
    documents: &FxHashMap<String, String>,
    mut source: impl FnMut(&str) -> Result<Option<Positions>, E>,
) -> Result<Option<Vec<Vec<Diagnostic>>>, E> {
    let mut positions = FxHashMap::default();
    for (uri, text) in documents {
        let Some(text_positions) = Positions::new(text) else {
            return Ok(None);
        };
        positions.insert(uri.clone(), text_positions);
    }
    let mut by_uri: FxHashMap<String, Vec<Diagnostic>> = FxHashMap::default();
    for uri in uris {
        // Requested documents must still belong to the acknowledged overlay.
        if !positions.contains_key(uri.as_str()) {
            return Ok(None);
        }
        by_uri.entry(uri.clone()).or_default();
    }
    for category in categories {
        for native in category {
            if native.file_name.is_empty() {
                return Ok(None);
            }
            let uri = path_to_file_uri(Path::new(native.file_name.as_str()));
            if let Some(diagnostics) = by_uri.get_mut(uri.as_str()) {
                let Some(diagnostic) = convert(native, &uri, &mut positions, &mut source)? else {
                    return Ok(None);
                };
                diagnostics.push(diagnostic);
            }
        }
    }
    // Preserve requested order and duplicates, including successful empties.
    Ok(uris
        .iter()
        .map(|uri| by_uri.get(uri.as_str()).cloned())
        .collect())
}

fn convert<E>(
    native: &NativeDiagnostic,
    uri: &str,
    positions: &mut FxHashMap<String, Positions>,
    source: &mut impl FnMut(&str) -> Result<Option<Positions>, E>,
) -> Result<Option<Diagnostic>, E> {
    let mut related = Vec::with_capacity(native.related_information.len());
    for info in &native.related_information {
        if info.file_name.is_empty() {
            return Ok(None);
        }
        let uri = path_to_file_uri(Path::new(info.file_name.as_str()));
        // Only an exact same-snapshot provider may supply an unmirrored source.
        // Cache its positions once inside the existing conversion, without
        // another diagnostic walk or a read from the later filesystem state.
        if !positions.contains_key(uri.as_str()) {
            let Some(owned_positions) = source(&uri)? else {
                return Ok(None);
            };
            positions.insert(String::from(uri.as_str()), owned_positions);
        }
        let Some(range) = positions
            .get(uri.as_str())
            .and_then(|p| p.range(info.pos, info.end))
        else {
            return Ok(None);
        };
        let Ok(uri) = Uri::from_str(&uri) else {
            return Ok(None);
        };
        related.push(DiagnosticRelatedInformation {
            location: Location::new(uri, range),
            message: info.text.as_str().into(),
        });
    }
    let Some(range) = positions
        .get(uri)
        .and_then(|p| p.range(native.pos, native.end))
    else {
        return Ok(None);
    };
    Ok(Some(Diagnostic {
        range,
        severity: Some(severity(native.category, native.code)),
        code: Some(NumberOrString::Number(native.code)),
        source: Some("ts".into()),
        message: message(native).as_str().into(),
        related_information: (!related.is_empty()).then_some(related),
        ..Default::default()
    }))
}

fn severity(category: u8, code: i32) -> DiagnosticSeverity {
    match category {
        0 => DiagnosticSeverity::WARNING,
        2 => DiagnosticSeverity::HINT,
        3 => DiagnosticSeverity::INFORMATION,
        _ if matches!(code, 6133 | 6138 | 6192 | 6196 | 7027 | 7028 | 7029 | 7030) => {
            DiagnosticSeverity::WARNING
        }
        _ => DiagnosticSeverity::ERROR,
    }
}

fn message(native: &NativeDiagnostic) -> String {
    let mut result = String::default();
    let mut first = true;
    let mut pending = vec![(native, 0)];
    while let Some((node, depth)) = pending.pop() {
        if !first {
            result.push('\n');
        }
        first = false;
        for _ in 0..depth {
            result.push_str("  ");
        }
        result.push_str(&node.text);
        pending.extend(
            node.message_chain
                .iter()
                .rev()
                .map(|child| (child, depth + 1)),
        );
    }
    result
}
