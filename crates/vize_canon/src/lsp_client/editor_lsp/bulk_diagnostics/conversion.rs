//! Native categories projected using the pinned LSP converter's semantics.

use super::positions::Positions;
use crate::file_uri::{file_uri_to_path, path_to_file_uri};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Uri,
};
use serde::Deserialize;
use std::{path::Path, str::FromStr};
use vize_l0::{FxHashMap, FxHashSet, String};

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

pub(super) fn requested_members_are_present(uris: &[String], names: &[String]) -> bool {
    let members = names
        .iter()
        .map(|name| Path::new(name.as_str()))
        .collect::<FxHashSet<_>>();
    uris.iter().all(|uri| {
        file_uri_to_path(uri).is_some_and(|path| {
            members.contains(path.as_path()) && path_to_file_uri(&path) == uri.as_str()
        })
    })
}

pub(super) fn project_diagnostics(
    categories: &[Vec<NativeDiagnostic>],
    uris: &[String],
    documents: &FxHashMap<String, String>,
) -> Option<Vec<Vec<Diagnostic>>> {
    let mut positions = FxHashMap::default();
    for (uri, text) in documents {
        positions.insert(uri.clone(), Positions::new(text)?);
    }
    let mut by_uri: FxHashMap<String, Vec<Diagnostic>> = FxHashMap::default();
    for uri in uris {
        if !positions.contains_key(uri.as_str()) {
            return None;
        }
        by_uri.entry(uri.clone()).or_default();
    }
    for category in categories {
        for native in category {
            // Fileless rows cannot be assigned to a per-document pull result.
            if native.file_name.is_empty() {
                return None;
            }
            let uri = path_to_file_uri(Path::new(native.file_name.as_str()));
            if let Some(diagnostics) = by_uri.get_mut(uri.as_str()) {
                diagnostics.push(convert(native, &uri, &positions)?);
            }
        }
    }
    // Preserve requested order and duplicates, including successful empties.
    uris.iter()
        .map(|uri| by_uri.get(uri.as_str()).cloned())
        .collect()
}

fn convert(
    native: &NativeDiagnostic,
    uri: &str,
    positions: &FxHashMap<String, Positions>,
) -> Option<Diagnostic> {
    let mut related = Vec::with_capacity(native.related_information.len());
    for info in &native.related_information {
        if info.file_name.is_empty() {
            return None;
        }
        let uri = path_to_file_uri(Path::new(info.file_name.as_str()));
        // Reading disk after freezing the snapshot would not establish that
        // text belongs to it. Unmirrored related sources require whole fallback.
        let range = positions.get(uri.as_str())?.range(info.pos, info.end)?;
        related.push(DiagnosticRelatedInformation {
            location: Location::new(Uri::from_str(&uri).ok()?, range),
            message: info.text.as_str().into(),
        });
    }
    Some(Diagnostic {
        range: positions.get(uri)?.range(native.pos, native.end)?,
        severity: Some(severity(native.category, native.code)),
        code: Some(NumberOrString::Number(native.code)),
        source: Some("ts".into()),
        message: message(native).as_str().into(),
        related_information: (!related.is_empty()).then_some(related),
        // Canon's existing pull capabilities advertise related information,
        // but no tag value set or Visual Studio extensions.
        ..Default::default()
    })
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
