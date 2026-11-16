//! Relative import specifiers in the checker document point at the SFC.
//!
//! Corsa reads `active.patina.ts` from a session directory, not from the
//! Vue file's folder. A specifier that starts with `.` is rewritten to an
//! absolute path next to `filename`, and generated mapping rows move with
//! the new bytes.

use std::path::Path;

use vize_l0::String as VizeString;

use super::document::TypeAwareDocument;
use crate::linter::corsa_session::escape_specifier;

pub(super) fn absolutize_relative_imports(document: &mut TypeAwareDocument, filename: &str) {
    let Some(parent) = source_directory(filename) else {
        return;
    };
    let mut text = document.content.clone();
    let mut edits = Vec::new();
    let mut from = 0usize;
    while let Some((start, end)) = next_relative_specifier(&text, from) {
        let relative = text.get(start..end).unwrap_or_default();
        let resolved = parent.join(relative);
        let absolute = display_path(&vize_l0::path::canonicalize_non_verbatim(&normalize_path(
            &resolved,
        )));
        // `allowImportingTsExtensions` covers relative specifiers only. An
        // absolute `api.ts` is an error module (`any`); the extensionless
        // path still resolves under `moduleResolution: bundler` (#12948).
        let absolute = strip_implementation_extension(absolute);
        let quote = text.as_bytes().get(start - 1).copied().unwrap_or(b'\'');
        edits.push((start, end, escape_specifier(&absolute, quote)));
        from = end;
    }
    for (start, end, absolute) in edits.into_iter().rev() {
        document
            .mapping
            .note_generated_replacement(start, end - start, absolute.len());
        text.replace_range(start..end, &absolute);
    }
    document.content = text;
}

fn source_directory(filename: &str) -> Option<std::path::PathBuf> {
    let path = Path::new(filename);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    path.parent().map(Path::to_path_buf)
}

/// Specifier byte range inside `from "./mod"` or `from './mod'`.
fn next_relative_specifier(content: &str, from: usize) -> Option<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut index = from;
    while index + 6 < bytes.len() {
        if is_from_keyword(bytes, index) {
            let mut cursor = index + 4;
            while bytes.get(cursor) == Some(&b' ') {
                cursor += 1;
            }
            if let Some(&quote @ (b'"' | b'\'')) = bytes.get(cursor) {
                let spec_start = cursor + 1;
                if content
                    .get(spec_start..)
                    .unwrap_or_default()
                    .starts_with('.')
                    && let Some(relative_end) = content
                        .get(spec_start..)
                        .unwrap_or_default()
                        .find(quote as char)
                {
                    return Some((spec_start, spec_start + relative_end));
                }
            }
        }
        index += 1;
    }
    None
}

fn is_from_keyword(bytes: &[u8], index: usize) -> bool {
    if !bytes.get(index..).unwrap_or_default().starts_with(b"from") {
        return false;
    }
    let before = index.checked_sub(1).and_then(|at| bytes.get(at).copied());
    let after = bytes.get(index + 4).copied();
    !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
}

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn display_path(path: &Path) -> VizeString {
    VizeString::from(path.to_string_lossy().replace('\\', "/"))
}

/// Drop `.ts` / `.tsx` / `.mts` / `.cts` from a checker path. `.d.ts` stays,
/// and `.js` stays so TypeScript can map it back onto the implementation.
fn strip_implementation_extension(path: VizeString) -> VizeString {
    let text = path.as_str();
    if text.ends_with(".d.ts") || text.ends_with(".d.mts") || text.ends_with(".d.cts") {
        return path;
    }
    for suffix in [".mts", ".cts", ".tsx", ".ts"] {
        if let Some(stripped) = text.strip_suffix(suffix) {
            return VizeString::from(stripped);
        }
    }
    path
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use vize_canon::virtual_ts::ProjectionMapping;
    use vize_canon::virtual_ts::VizeMapping;
    use vize_l0::String as VizeString;

    use super::absolutize_relative_imports;
    use crate::linter::native_type_aware::document::TypeAwareDocument;

    #[test]
    fn a_relative_import_becomes_absolute_and_later_rows_move() {
        let source = "import { useAnnouncer } from \"./announcer-runtime.ts\";\nconst y = 1;\n";
        let marker = source.find("const y").expect("marker");
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![VizeMapping::new(
                marker..marker + 1,
                4..5,
            )]),
        };
        absolutize_relative_imports(&mut document, "/pkg/src/Widget.vue");
        assert_eq!(
            document.content.as_str(),
            "import { useAnnouncer } from \"/pkg/src/announcer-runtime\";\nconst y = 1;\n"
        );
        let generated = document.mapping.to_generated(4).expect("moved row");
        assert_eq!(
            &document.content[generated..generated + "const y".len()],
            "const y"
        );
    }

    #[test]
    fn declaration_and_javascript_specifiers_keep_their_extensions() {
        let source =
            "import type { Props } from \"./props.d.ts\";\nexport { api } from './api.js';\n";
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![]),
        };
        absolutize_relative_imports(&mut document, "/pkg/src/Widget.vue");
        assert_eq!(
            document.content.as_str(),
            "import type { Props } from \"/pkg/src/props.d.ts\";\nexport { api } from '/pkg/src/api.js';\n"
        );
    }

    #[test]
    fn existing_files_and_directories_use_checker_compatible_absolute_paths() {
        let package = tempdir().expect("temporary package");
        let source_dir = package.path().join("src");
        fs::create_dir_all(source_dir.join("lib")).expect("library directory");
        fs::write(source_dir.join("api.ts"), "export const value = 1;\n")
            .expect("TypeScript module");
        fs::write(
            source_dir.join("lib/index.ts"),
            "export const nested = 2;\n",
        )
        .expect("directory module");

        let source = "import { value } from './api.ts';\nimport { nested } from './lib';\n";
        let mut document = TypeAwareDocument {
            content: VizeString::from(source),
            mapping: ProjectionMapping::from_spans(vec![]),
        };
        absolutize_relative_imports(
            &mut document,
            source_dir.join("Panel.vue").to_str().expect("UTF-8 path"),
        );

        let normalized = document.content.replace('\\', "/");
        let normalized_source_dir = vize_l0::path::canonicalize_non_verbatim(&source_dir)
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(
            normalized,
            format!(
                "import {{ value }} from '{normalized_source_dir}/api';\n\
                 import {{ nested }} from '{normalized_source_dir}/lib';\n"
            )
        );
    }
}
