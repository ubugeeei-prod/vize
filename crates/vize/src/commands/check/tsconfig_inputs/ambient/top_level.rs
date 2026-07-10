pub(super) fn has_top_level_import_or_export(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    let mut brace_depth = 0usize;
    let mut at_statement_start = true;

    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'\'' | b'"' | b'`' => {
                index = skip_quoted(bytes, index);
            }
            b'{' => {
                brace_depth += 1;
                at_statement_start = false;
                index += 1;
            }
            b'}' => {
                brace_depth = brace_depth.saturating_sub(1);
                at_statement_start = brace_depth == 0;
                index += 1;
            }
            b';' if brace_depth == 0 => {
                at_statement_start = true;
                index += 1;
            }
            ch if ch.is_ascii_whitespace() => {
                index += 1;
            }
            _ if brace_depth == 0 && at_statement_start => {
                if starts_with_statement_keyword(bytes, index, b"import")
                    || starts_with_statement_keyword(bytes, index, b"export")
                {
                    return true;
                }
                at_statement_start = false;
                index = skip_token(bytes, index);
            }
            _ => {
                index += 1;
            }
        }
    }

    false
}

fn starts_with_statement_keyword(bytes: &[u8], index: usize, keyword: &[u8]) -> bool {
    if !bytes[index..].starts_with(keyword) {
        return false;
    }
    let after = index + keyword.len();
    if bytes
        .get(after)
        .is_some_and(|ch| is_identifier_continue(*ch))
    {
        return false;
    }

    match keyword {
        b"import" => bytes
            .get(after)
            .is_some_and(|ch| ch.is_ascii_whitespace() || matches!(ch, b'{' | b'*' | b'\'' | b'"')),
        b"export" => bytes
            .get(after)
            .is_some_and(|ch| ch.is_ascii_whitespace() || matches!(ch, b'{' | b'*' | b'=')),
        _ => false,
    }
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> usize {
    while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
        index += 1;
    }
    (index + 2).min(bytes.len())
}

fn skip_quoted(bytes: &[u8], mut index: usize) -> usize {
    let quote = bytes[index];
    index += 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                index = (index + 2).min(bytes.len());
            }
            ch if ch == quote => {
                return index + 1;
            }
            _ => {
                index += 1;
            }
        }
    }
    bytes.len()
}

fn skip_token(bytes: &[u8], mut index: usize) -> usize {
    if !is_identifier_start(bytes[index]) {
        return index + 1;
    }
    index += 1;
    while index < bytes.len() && is_identifier_continue(bytes[index]) {
        index += 1;
    }
    index
}

fn is_identifier_start(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || ch == b'_' || ch == b'$'
}

fn is_identifier_continue(ch: u8) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::has_top_level_import_or_export;

    #[test]
    fn nested_exports_do_not_make_ambient_declarations_external_modules() {
        assert!(!has_top_level_import_or_export(
            "declare module \"vue\" {\n  export interface GlobalComponents {}\n}\n",
        ));
        assert!(!has_top_level_import_or_export(
            "declare global { export interface Window { appVersion: string } }\n",
        ));
        assert!(!has_top_level_import_or_export(
            "type VueComponent = import('vue').Component;\n",
        ));
    }

    #[test]
    fn top_level_imports_and_exports_make_declarations_external_modules() {
        assert!(has_top_level_import_or_export(
            "import \"vue\";\ndeclare module \"vue\" { export interface GlobalComponents {} }\n",
        ));
        assert!(has_top_level_import_or_export(
            "declare namespace Local { export interface Thing {} }\nexport type Named = string;\n",
        ));
        assert!(has_top_level_import_or_export("export{};\n"));
        assert!(has_top_level_import_or_export(
            "/* comment */\nexport enum GeneratedKind { A = 'A' }\n",
        ));
    }
}
