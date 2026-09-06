use std::path::Path;

use tower_lsp::lsp_types::DocumentLink;

use super::DocumentLinkService;

/// Collect import statement links from script content.
pub(super) fn collect_import_links(
    script: &str,
    base_offset: usize,
    full_content: &str,
    base_path: Option<&Path>,
    links: &mut Vec<DocumentLink>,
) {
    let mut pos = 0;

    while let Some(start) = next_module_keyword(script, pos) {
        let stmt_end = script[start..]
            .find([';', '\n'])
            .map(|i| start + i)
            .unwrap_or(script.len());

        let stmt = &script[start..stmt_end];

        if let Some((path, rel_start, rel_end)) = extract_import_path(stmt)
            && (path.starts_with('.') || path.starts_with('/'))
            && let Some(target) = DocumentLinkService::resolve_path(&path, base_path)
        {
            let abs_start = base_offset + start + rel_start;
            let abs_end = base_offset + start + rel_end;
            links.push(DocumentLinkService::create_link(
                full_content,
                abs_start,
                abs_end,
                target,
            ));
        }

        pos = stmt_end + 1;
    }
}

/// Extract import path from an import/export statement.
/// Returns (path, start, end_offset) where offsets are relative to stmt start.
fn extract_import_path(stmt: &str) -> Option<(String, usize, usize)> {
    if let Some(from_pos) = stmt.find(" from ") {
        let after_from = &stmt[from_pos + 6..];
        return extract_string_literal(after_from)
            .map(|(path, s, e)| (path, from_pos + 6 + s, from_pos + 6 + e));
    }

    if stmt.starts_with("import ") || stmt.starts_with("import\t") {
        let after_import = &stmt[7..];
        let trimmed = after_import.trim_start();
        if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            let ws_len = after_import.len() - trimmed.len();
            return extract_string_literal(trimmed)
                .map(|(path, s, e)| (path, 7 + ws_len + s, 7 + ws_len + e));
        }
    }

    None
}

/// Extract string literal from text.
/// Returns (content, start, end_offset) where offsets include quotes.
pub(super) fn extract_string_literal(text: &str) -> Option<(String, usize, usize)> {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let quote = bytes[0] as char;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == quote as u8 && (i == 1 || bytes[i - 1] != b'\\') {
            let content = text[1..i].to_string();
            return Some((content, 0, i + 1));
        }
        i += 1;
    }

    None
}

fn next_module_keyword(script: &str, mut pos: usize) -> Option<usize> {
    let bytes = script.as_bytes();

    while pos < bytes.len() {
        match bytes[pos] {
            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                pos = skip_line_comment(bytes, pos + 2);
            }
            b'/' if bytes.get(pos + 1) == Some(&b'*') => {
                pos = skip_block_comment(bytes, pos + 2);
            }
            b'\'' | b'"' | b'`' => {
                pos = skip_quoted(bytes, pos, bytes[pos]);
            }
            b'i' if keyword_at(bytes, pos, b"import") => return Some(pos),
            b'e' if keyword_at(bytes, pos, b"export") => return Some(pos),
            _ => pos += 1,
        }
    }

    None
}

fn skip_line_comment(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    pos.saturating_add(1).min(bytes.len())
}

fn skip_block_comment(bytes: &[u8], mut pos: usize) -> usize {
    while pos + 1 < bytes.len() {
        if bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
            return pos + 2;
        }
        pos += 1;
    }
    bytes.len()
}

fn skip_quoted(bytes: &[u8], mut pos: usize, quote: u8) -> usize {
    pos += 1;

    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos = pos.saturating_add(2).min(bytes.len()),
            byte if byte == quote => return pos + 1,
            _ => pos += 1,
        }
    }

    bytes.len()
}

fn keyword_at(bytes: &[u8], pos: usize, keyword: &[u8]) -> bool {
    bytes.get(pos..pos + keyword.len()) == Some(keyword)
        && pos
            .checked_sub(1)
            .is_none_or(|prev| !is_ident_byte(bytes[prev]))
        && bytes
            .get(pos + keyword.len())
            .is_none_or(|next| !is_ident_byte(*next))
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::{extract_import_path, extract_string_literal, next_module_keyword};

    #[test]
    fn extracts_import_paths() {
        let stmt = r#"import { ref } from "./utils""#;
        let (path, _, _) = extract_import_path(stmt).unwrap();
        assert_eq!(path, "./utils");

        let stmt = r#"import "./styles.css""#;
        let (path, _, _) = extract_import_path(stmt).unwrap();
        assert_eq!(path, "./styles.css");

        let stmt = r#"export { foo } from './foo'"#;
        let (path, _, _) = extract_import_path(stmt).unwrap();
        assert_eq!(path, "./foo");
    }

    #[test]
    fn extracts_string_literals() {
        let (content, start, end) = extract_string_literal(r#""hello""#).unwrap();
        assert_eq!(content, "hello");
        assert_eq!(start, 0);
        assert_eq!(end, 7);

        let (content, _, _) = extract_string_literal("'world'").unwrap();
        assert_eq!(content, "world");
    }

    #[test]
    fn skips_keywords_inside_comments_and_strings() {
        let script = concat!(
            "// import Ghost from './ghost'\n",
            "const note = \"export { Hidden } from './hidden'\"\n",
            "const tpl = `import Other from './other'`\n",
            "import Real from './real'\n",
        );

        assert_eq!(next_module_keyword(script, 0), script.find("import Real"));
    }
}
