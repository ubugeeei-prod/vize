use std::path::Path;

use vize_s0::{String, ToCompactString};

pub(super) fn rewrite_relative_import_types(type_annotation: &str, source_dir: &Path) -> String {
    let bytes = type_annotation.as_bytes();
    let mut out = String::with_capacity(type_annotation.len());
    let mut i = 0usize;

    while i < bytes.len() {
        let import_prefix = if type_annotation[i..].starts_with("import('") {
            Some('\'')
        } else if type_annotation[i..].starts_with("import(\"") {
            Some('"')
        } else {
            None
        };

        let Some(quote) = import_prefix else {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        };

        out.push_str("import(");
        out.push(quote);
        i += 8;

        let start = i;
        while i < bytes.len() && bytes[i] != quote as u8 {
            i += 1;
        }

        let specifier = &type_annotation[start..i];
        out.push_str(&rewrite_relative_specifier(specifier, source_dir));

        if i < bytes.len() {
            out.push(quote);
            i += 1;
        }
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
