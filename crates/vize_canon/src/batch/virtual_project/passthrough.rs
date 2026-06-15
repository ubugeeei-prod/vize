//! Discovering relative JSON modules imported by a source file so they can be
//! mirrored into the virtual project, keeping TypeScript's module resolution
//! happy. Includes a lightweight byte-level scanner for `import`/`from`
//! specifiers.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashSet, String as CompactString, ToCompactString, cstr};

use super::build::mirrored_virtual_path;

pub(super) fn collect_passthrough_json_modules(
    path: &Path,
    content: &str,
    project_root: &Path,
    virtual_root: &Path,
) -> Vec<(PathBuf, PathBuf)> {
    let Some(dir) = path.parent() else {
        return Vec::new();
    };

    let mut seen = FxHashSet::default();
    let mut files = Vec::new();
    for specifier in extract_relative_module_specifiers(content) {
        let Some(original_path) = resolve_relative_json_module(dir, &specifier) else {
            continue;
        };
        let Ok(virtual_path) = mirrored_virtual_path(project_root, virtual_root, &original_path)
        else {
            continue;
        };
        if seen.insert(virtual_path.clone()) {
            files.push((virtual_path, original_path));
        }
    }
    files
}

fn extract_relative_module_specifiers(source: &str) -> Vec<CompactString> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut specifiers = Vec::new();
    let mut i = 0;

    while i < len {
        let keyword_len = if matches_keyword(bytes, i, b"from") {
            4
        } else if matches_keyword(bytes, i, b"import") {
            6
        } else {
            i += 1;
            continue;
        };

        let mut j = i + keyword_len;
        while j < len && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j < len && bytes[j] == b'(' {
            j += 1;
            while j < len && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
        }

        if j < len && (bytes[j] == b'"' || bytes[j] == b'\'') {
            let quote = bytes[j];
            let start = j + 1;
            let mut k = start;
            while k < len && bytes[k] != quote {
                k += 1;
            }
            if k < len {
                let specifier = &source[start..k];
                if is_relative_specifier(specifier) {
                    specifiers.push(specifier.to_compact_string());
                }
                i = k + 1;
                continue;
            }
        }

        i += keyword_len;
    }

    specifiers
}

fn matches_keyword(bytes: &[u8], at: usize, keyword: &[u8]) -> bool {
    if at + keyword.len() > bytes.len() || &bytes[at..at + keyword.len()] != keyword {
        return false;
    }
    let before_ok = at == 0 || !is_identifier_byte(bytes[at - 1]);
    let after = at + keyword.len();
    let after_ok = after >= bytes.len() || !is_identifier_byte(bytes[after]);
    before_ok && after_ok
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
}

fn resolve_relative_json_module(dir: &Path, specifier: &str) -> Option<PathBuf> {
    let base = dir.join(specifier);

    if specifier.ends_with(".json") && base.is_file() {
        return Some(normalize_existing_path(&base));
    }

    let candidate = append_json_extension(&base);
    if candidate.is_file() {
        return Some(normalize_existing_path(&candidate));
    }

    let candidate = base.join("index.json");
    if candidate.is_file() {
        return Some(normalize_existing_path(&candidate));
    }

    None
}

fn append_json_extension(base: &Path) -> PathBuf {
    match base.file_name().and_then(|name| name.to_str()) {
        Some(name) => base.with_file_name(cstr!("{name}.json")),
        None => base.to_path_buf(),
    }
}

fn normalize_existing_path(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}
