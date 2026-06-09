//! Lightweight scanner for static and dynamic import statements in source text.

use vize_carton::{String, ToCompactString};

#[derive(Debug)]
pub(super) struct ImportEdge {
    pub specifier: String,
    pub kind: &'static str,
    pub locals: Vec<String>,
}

pub(super) fn extract_imports(source: &str) -> Vec<ImportEdge> {
    let mut imports = Vec::new();
    collect_static_imports(source, &mut imports);
    collect_dynamic_imports(source, &mut imports);
    imports
}

fn collect_static_imports(source: &str, imports: &mut Vec<ImportEdge>) {
    let mut offset = 0;
    while let Some(index) = source[offset..].find("import") {
        let import_start = offset + index;
        let clause_start = import_start + "import".len();
        if !is_word_boundary(source, import_start, clause_start) {
            offset = clause_start;
            continue;
        }

        let start = skip_whitespace(source, clause_start);
        if source[start..].starts_with('(') {
            offset = start + 1;
            continue;
        }
        if source[start..].starts_with('.') {
            offset = start + 1;
            continue;
        }

        if let Some((specifier, end)) = read_quoted(source, start) {
            imports.push(ImportEdge {
                specifier,
                kind: "import",
                locals: Vec::new(),
            });
            offset = end;
            continue;
        }

        let Some(from_index) = find_import_from(source, start) else {
            offset = clause_start;
            continue;
        };

        if let Some((specifier, end)) = read_quoted(source, from_index + "from".len()) {
            let clause = &source[start..from_index];
            imports.push(ImportEdge {
                specifier,
                kind: "import",
                locals: extract_import_locals(clause),
            });
            offset = end;
        } else {
            offset = from_index + "from".len();
        }
    }
}

fn collect_dynamic_imports(source: &str, imports: &mut Vec<ImportEdge>) {
    let mut offset = 0;
    while let Some(index) = source[offset..].find("import") {
        let import_start = offset + index;
        let import_end = import_start + "import".len();
        if !is_word_boundary(source, import_start, import_end) {
            offset = import_end;
            continue;
        }

        let start = skip_whitespace(source, import_end);
        if !source[start..].starts_with('(') {
            offset = import_end;
            continue;
        }

        if let Some((specifier, end)) = read_quoted(source, start + 1) {
            imports.push(ImportEdge {
                specifier,
                kind: "dynamic-import",
                locals: Vec::new(),
            });
            offset = end;
        } else {
            offset = start + 1;
        }
    }
}

pub(super) fn skip_whitespace(source: &str, mut index: usize) -> usize {
    let bytes = source.as_bytes();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    index
}

fn is_word_boundary(source: &str, start: usize, end: usize) -> bool {
    let bytes = source.as_bytes();
    let before = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_none_or(|byte| !is_identifier_byte(*byte));
    let after = bytes.get(end).is_none_or(|byte| !is_identifier_byte(*byte));
    before && after
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn find_import_from(source: &str, start: usize) -> Option<usize> {
    let mut offset = start;
    while let Some(index) = source[offset..].find("from") {
        let from_index = offset + index;
        let from_end = from_index + "from".len();
        if is_word_boundary(source, from_index, from_end) {
            return Some(from_index);
        }
        offset = from_end;
    }
    None
}

fn read_quoted(source: &str, start: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    let mut index = skip_whitespace(source, start);
    let quote = *bytes.get(index)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    index += 1;
    let value_start = index;
    while index < bytes.len() && bytes[index] != quote {
        index += 1;
    }
    if index >= bytes.len() {
        return None;
    }
    Some((String::from(&source[value_start..index]), index + 1))
}

fn extract_import_locals(clause: &str) -> Vec<String> {
    let mut locals = Vec::new();
    let trimmed = clause.trim();
    if trimmed.starts_with("type ") {
        return locals;
    }

    if let Some(default_name) = trimmed.split(',').next().map(str::trim)
        && is_identifier(default_name)
    {
        locals.push(default_name.to_compact_string());
    }

    if let Some(namespace_name) = trimmed.strip_prefix("* as ").map(str::trim)
        && is_identifier(namespace_name)
    {
        locals.push(namespace_name.to_compact_string());
    }

    if let Some(named_start) = trimmed.find('{')
        && let Some(named_end) = trimmed[named_start + 1..].find('}')
    {
        let named = &trimmed[named_start + 1..named_start + 1 + named_end];
        for part in named.split(',') {
            let part = part.trim();
            if part.starts_with("type ") {
                continue;
            }
            let local = part
                .rsplit_once(" as ")
                .map(|(_, local)| local)
                .unwrap_or(part);
            if is_identifier(local.trim()) {
                locals.push(local.trim().to_compact_string());
            }
        }
    }

    locals
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|char| char == '_' || char == '$' || char.is_ascii_alphanumeric())
}
