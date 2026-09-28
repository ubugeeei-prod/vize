//! Compile-time artifact macro import handling.

use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_span::GetSpan;
use vize_carton::{FxHashSet, String};
use vize_croquis::macros::macro_artifact_kind;

pub(super) fn collect_static_imports<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
    content: &str,
) -> String {
    let mut imports = String::default();

    for stmt in statements {
        if !matches!(stmt, Statement::ImportDeclaration(_)) {
            continue;
        }
        if is_artifact_macro_only_import(stmt) {
            continue;
        }

        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;
        let Some(import) = content.get(start..end) else {
            continue;
        };

        let removals = artifact_macro_import_removal_spans(stmt, content);
        if !removals.is_empty() {
            let mut cleaned = String::default();
            let mut cursor = start;
            for (remove_start, remove_end) in removals {
                cleaned.push_str(content.get(cursor..remove_start).unwrap_or_default());
                cursor = remove_end;
            }
            cleaned.push_str(content.get(cursor..end).unwrap_or_default());
            imports.push_str(cleaned.trim());
        } else {
            imports.push_str(import.trim());
        }
        imports.push('\n');
    }

    imports
}

pub(super) fn collect_artifact_macro_import_bindings<'a>(
    statements: impl Iterator<Item = &'a Statement<'a>>,
) -> FxHashSet<String> {
    let mut bindings = FxHashSet::default();

    for stmt in statements {
        let Statement::ImportDeclaration(import_decl) = stmt else {
            continue;
        };
        if import_decl.import_kind.is_type()
            || !is_known_artifact_macro_import_source(import_decl.source.value.as_str())
        {
            continue;
        }
        let Some(specifiers) = import_decl.specifiers.as_ref() else {
            continue;
        };
        for specifier in specifiers {
            if let Some(local) =
                artifact_macro_import_local_name(specifier, import_decl.source.value.as_str())
            {
                bindings.insert(local.into());
            }
        }
    }

    bindings
}

pub(super) fn is_artifact_macro_only_import(stmt: &Statement<'_>) -> bool {
    let Statement::ImportDeclaration(import_decl) = stmt else {
        return false;
    };
    if import_decl.import_kind.is_type()
        || !is_known_artifact_macro_import_source(import_decl.source.value.as_str())
    {
        return false;
    }
    let Some(specifiers) = import_decl.specifiers.as_ref() else {
        return false;
    };
    !specifiers.is_empty()
        && specifiers.iter().all(|specifier| {
            artifact_macro_import_local_name(specifier, import_decl.source.value.as_str()).is_some()
        })
}

/// Remove compile-time macros from an import that also carries runtime bindings.
/// Disjoint source spans keep the remaining import and its provenance byte-identical.
pub(super) fn artifact_macro_import_removal_spans(
    stmt: &Statement<'_>,
    content: &str,
) -> Vec<(usize, usize)> {
    let Statement::ImportDeclaration(import_decl) = stmt else {
        return Vec::new();
    };
    if import_decl.import_kind.is_type()
        || !is_known_artifact_macro_import_source(import_decl.source.value.as_str())
    {
        return Vec::new();
    }
    let Some(specifiers) = import_decl.specifiers.as_ref() else {
        return Vec::new();
    };
    if specifiers.len() < 2 {
        return Vec::new();
    }
    let named: Vec<_> = specifiers
        .iter()
        .filter_map(|specifier| {
            matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(_)).then_some((
                specifier.span(),
                artifact_macro_import_local_name(specifier, import_decl.source.value.as_str())
                    .is_some(),
            ))
        })
        .collect();
    if named.iter().all(|(_, is_macro)| !is_macro) {
        return Vec::new();
    }
    if named.iter().all(|(_, is_macro)| *is_macro) {
        let Some((first_span, _)) = named.first() else {
            return Vec::new();
        };
        let Some((last_span, _)) = named.last() else {
            return Vec::new();
        };
        let Some(before) = content.get(import_decl.span.start as usize..first_span.start as usize)
        else {
            return Vec::new();
        };
        let Some(after) = content.get(last_span.end as usize..import_decl.span.end as usize) else {
            return Vec::new();
        };
        let Some(open) = before.rfind('{') else {
            return Vec::new();
        };
        let Some(close) = after.find('}') else {
            return Vec::new();
        };
        let open = import_decl.span.start as usize + open;
        let close = last_span.end as usize + close + 1;
        let default_end = specifiers
            .first()
            .map(|specifier| specifier.span().end as usize);
        return vec![(default_end.filter(|&end| end < open).unwrap_or(open), close)];
    }

    let mut removals = Vec::new();
    let mut index = 0;
    while index < named.len() {
        let Some((_, is_macro)) = named.get(index) else {
            break;
        };
        if !is_macro {
            index += 1;
            continue;
        }
        let first = index;
        while named.get(index).is_some_and(|(_, is_macro)| *is_macro) {
            index += 1;
        }
        if let (Some((first_span, _)), Some((next_span, _))) = (named.get(first), named.get(index))
        {
            removals.push((first_span.start as usize, next_span.start as usize));
        } else if let (Some((previous_span, _)), Some((last_span, _))) = (
            first.checked_sub(1).and_then(|i| named.get(i)),
            named.last(),
        ) {
            removals.push((previous_span.end as usize, last_span.end as usize));
        }
    }
    removals
}

fn artifact_macro_import_local_name<'a>(
    specifier: &'a ImportDeclarationSpecifier<'a>,
    source: &str,
) -> Option<&'a str> {
    let ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier else {
        return None;
    };
    if spec.import_kind.is_type() {
        return None;
    }
    let imported = spec.imported.name().as_str();
    let local = spec.local.name.as_str();
    if imported != local
        || macro_artifact_kind(imported).is_none()
        || (source == "#imports" && imported != "definePageMeta")
    {
        return None;
    }
    Some(local)
}

fn is_known_artifact_macro_import_source(source: &str) -> bool {
    matches!(source, "@typed-router" | "#imports")
}
