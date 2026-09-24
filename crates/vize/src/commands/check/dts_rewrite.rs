use std::path::Path;

use vize_s0::{String, ToCompactString};

pub(super) fn rewrite_relative_import_types(type_annotation: &str, source_dir: &Path) -> String {
    rewrite_import_type_specifiers(type_annotation, |specifier| {
        rewrite_relative_specifier(specifier, source_dir)
    })
}

/// Rewrite the specifier of every `import('…')` / `import("…")` type in
/// `type_annotation` through `rewrite`, copying everything else verbatim.
pub(super) fn rewrite_import_type_specifiers(
    type_annotation: &str,
    mut rewrite: impl FnMut(&str) -> String,
) -> String {
    let mut out = String::with_capacity(type_annotation.len());
    let mut rest = type_annotation;

    while !rest.is_empty() {
        let quote = if rest.starts_with("import('") {
            '\''
        } else if rest.starts_with("import(\"") {
            '"'
        } else {
            let mut chars = rest.chars();
            if let Some(ch) = chars.next() {
                out.push(ch);
            }
            rest = chars.as_str();
            continue;
        };

        out.push_str("import(");
        out.push(quote);
        let after = rest.get("import('".len()..).unwrap_or_default();
        let (specifier, tail) = after
            .split_once(quote)
            .map_or((after, None), |(specifier, tail)| (specifier, Some(tail)));
        out.push_str(&rewrite(specifier));
        rest = match tail {
            Some(tail) => {
                out.push(quote);
                tail
            }
            None => "",
        };
    }

    out
}

pub(super) fn rewrite_relative_specifier(specifier: &str, source_dir: &Path) -> String {
    if !specifier.starts_with("./") && !specifier.starts_with("../") {
        return specifier.to_compact_string();
    }

    normalize_path(&source_dir.join(specifier))
}

fn normalize_path(path: &Path) -> String {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized.to_string_lossy().to_compact_string()
}
