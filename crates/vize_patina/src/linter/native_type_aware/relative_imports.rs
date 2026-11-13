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
        let absolute = checker_specifier(&resolved, resolved.canonicalize().ok());
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

/// The absolute specifier the checker resolves for `resolved`, given its
/// canonical path when it exists on disk.
///
/// Windows `canonicalize` returns a verbatim path (`\\?\D:\app\src\api.ts`),
/// which TypeScript reads as `//?/D:/app/src/api.ts` and cannot resolve, so
/// every import of an existing file or directory was `any` there (#12952).
/// The prefix is dropped wherever the path came from.
fn checker_specifier(resolved: &Path, canonical: Option<std::path::PathBuf>) -> VizeString {
    let path = canonical.unwrap_or_else(|| normalize_path(resolved));
    let absolute = display_path(&vize_l0::path::strip_verbatim_prefix(path));
    // `allowImportingTsExtensions` covers relative specifiers only. An
    // absolute `api.ts` is an error module (`any`); the extensionless
    // path still resolves under `moduleResolution: bundler` (#12948).
    strip_implementation_extension(absolute)
}

fn source_directory(filename: &str) -> Option<std::path::PathBuf> {
    let path = vize_l0::path::strip_verbatim_prefix(std::path::PathBuf::from(filename));
    let path = if path.is_absolute() {
        path
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
    use vize_canon::virtual_ts::ProjectionMapping;
    use vize_canon::virtual_ts::VizeMapping;
    use vize_l0::String as VizeString;

    use super::{absolutize_relative_imports, checker_specifier};
    use crate::linter::native_type_aware::document::TypeAwareDocument;
    use std::path::{Path, PathBuf};

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

    /// Windows `canonicalize` spellings of an existing file and directory
    /// (#12952), which this host cannot produce itself.
    #[test]
    fn a_verbatim_canonical_path_resolves_like_the_plain_one() {
        let resolved = Path::new("D:/app/src/./api.ts");
        let cases = [
            (r"\\?\D:\app\src\api.ts", "D:/app/src/api"),
            (r"\\?\D:\app\src\lib", "D:/app/src/lib"),
            (
                r"\\?\UNC\server\share\app\src\api.ts",
                "//server/share/app/src/api",
            ),
            ("//?/D:/app/src/props.d.ts", "D:/app/src/props.d.ts"),
        ];
        for (canonical, expected) in cases {
            assert_eq!(
                checker_specifier(resolved, Some(PathBuf::from(canonical))).as_str(),
                expected,
                "{canonical}"
            );
        }
        // A path that does not exist keeps the lexical spelling.
        assert_eq!(checker_specifier(resolved, None).as_str(), "D:/app/src/api");
    }
}
