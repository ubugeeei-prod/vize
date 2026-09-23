//! Relative import specifiers in the checker document point at the SFC.
//!
//! Corsa reads `active.patina.ts` from a session directory, not from the
//! Vue file's folder. A specifier that starts with `.` is rewritten to an
//! absolute path next to `filename`, and generated mapping rows move with
//! the new bytes.

use std::path::Path;

use vize_s0::String as VizeString;

use super::document::TypeAwareDocument;

pub(super) fn absolutize_relative_imports(document: &mut TypeAwareDocument, filename: &str) {
    let Some(parent) = source_directory(filename) else {
        return;
    };
    let mut text = document.content.clone();
    let mut edits = Vec::new();
    let mut from = 0usize;
    while let Some((start, end)) = next_relative_specifier(&text, from) {
        let relative = &text[start..end];
        let resolved = parent.join(relative);
        let absolute = resolved
            .canonicalize()
            .map(|path| display_path(&path))
            .unwrap_or_else(|_| display_path(&normalize_path(&resolved)));
        edits.push((start, end, absolute));
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
            while cursor < bytes.len() && bytes[cursor] == b' ' {
                cursor += 1;
            }
            if cursor < bytes.len() && (bytes[cursor] == b'"' || bytes[cursor] == b'\'') {
                let quote = bytes[cursor];
                let spec_start = cursor + 1;
                if content[spec_start..].starts_with('.')
                    && let Some(relative_end) = content[spec_start..].find(quote as char)
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
    if !bytes[index..].starts_with(b"from") {
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

#[cfg(test)]
mod tests {
    use vize_canon::virtual_ts::ProjectionMapping;
    use vize_canon::virtual_ts::VizeMapping;
    use vize_s0::String as VizeString;

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
            "import { useAnnouncer } from \"/pkg/src/announcer-runtime.ts\";\nconst y = 1;\n"
        );
        let generated = document.mapping.to_generated(4).expect("moved row");
        assert_eq!(
            &document.content[generated..generated + "const y".len()],
            "const y"
        );
    }
}
