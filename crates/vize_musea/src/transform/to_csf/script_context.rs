use crate::types::ArtDescriptor;
use std::collections::BTreeSet;
use vize_carton::{String, ToCompactString};

#[derive(Default)]
pub(super) struct CsfScriptContext {
    pub(super) imports: String,
    pub(super) setup_code: String,
    pub(super) setup_bindings: Vec<String>,
}

pub(super) fn collect_script_context(art: &ArtDescriptor<'_>) -> CsfScriptContext {
    let Some(script) = &art.script_setup else {
        return CsfScriptContext::default();
    };

    let parsed = vize_croquis::script_parser::parse_script_setup(script.content);
    let mut ranges = Vec::<(usize, usize)>::new();
    let mut imports = String::default();

    for import in &parsed.import_statements {
        let range = (import.start as usize, import.end as usize);
        if let Some(source) = script.content.get(range.0..range.1) {
            imports.push_str(source.trim());
            imports.push('\n');
            ranges.push(range);
        }
    }

    if let Some(call) = parsed.macros.define_art_call() {
        ranges.push(expand_statement_range(
            script.content,
            call.start as usize,
            call.end as usize,
        ));
    }

    let references = collect_template_identifiers(art);
    let mut setup_bindings = parsed
        .bindings
        .iter()
        .filter(|(name, _)| references.contains(*name))
        .map(|(name, _)| name.to_compact_string())
        .collect::<Vec<_>>();
    setup_bindings.sort();

    CsfScriptContext {
        imports,
        setup_code: remove_ranges(script.content, ranges),
        setup_bindings,
    }
}

fn expand_statement_range(source: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut expanded_end = end.min(bytes.len());
    while expanded_end < bytes.len()
        && bytes[expanded_end].is_ascii_whitespace()
        && bytes[expanded_end] != b'\n'
    {
        expanded_end += 1;
    }
    if expanded_end < bytes.len() && bytes[expanded_end] == b';' {
        expanded_end += 1;
    }
    (start.min(bytes.len()), expanded_end)
}

fn remove_ranges(source: &str, mut ranges: Vec<(usize, usize)>) -> String {
    ranges.sort_unstable();

    let mut output = String::default();
    let mut cursor = 0;
    for (start, end) in ranges {
        if start > cursor
            && let Some(chunk) = source.get(cursor..start)
        {
            output.push_str(chunk);
        }
        cursor = cursor.max(end);
    }
    if cursor < source.len()
        && let Some(chunk) = source.get(cursor..source.len())
    {
        output.push_str(chunk);
    }

    normalize_setup_code(&output)
}

fn normalize_setup_code(source: &str) -> String {
    let lines = source
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty() && line.trim() != ";")
        .collect::<Vec<_>>();

    lines.join("\n").trim().to_compact_string()
}

fn collect_template_identifiers(art: &ArtDescriptor<'_>) -> BTreeSet<String> {
    let mut identifiers = BTreeSet::new();
    for variant in &art.variants {
        collect_identifiers_into(variant.template, &mut identifiers);
    }
    identifiers
}

fn collect_identifiers_into(source: &str, identifiers: &mut BTreeSet<String>) {
    let bytes = source.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        if !is_identifier_start(bytes[cursor]) {
            cursor += 1;
            continue;
        }

        let start = cursor;
        cursor += 1;
        while cursor < bytes.len() && is_identifier_continue(bytes[cursor]) {
            cursor += 1;
        }

        if let Some(identifier) = source.get(start..cursor) {
            identifiers.insert(identifier.to_compact_string());
        }
    }
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic()
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}
