//! Script setup extraction for Storybook CSF output.

use vize_carton::String;
use vize_croquis::script_parser::parse_script_setup;

#[derive(Debug, Default)]
pub(super) struct ScriptSetupCsf {
    pub imports: String,
    pub setup_body: String,
    pub setup_bindings: Vec<String>,
    pub component_bindings: Vec<String>,
}

pub(super) fn extract_script_setup_for_csf(content: &str) -> ScriptSetupCsf {
    let parsed = parse_script_setup(content);
    let mut import_ranges = parsed
        .import_statements
        .iter()
        .map(|import| (import.start as usize, import.end as usize))
        .collect::<Vec<_>>();
    import_ranges.sort_unstable();

    let mut removal_ranges = import_ranges.clone();
    if let Some(define_art) = parsed.macros.define_art_call() {
        removal_ranges.push((
            define_art.start as usize,
            extend_through_semicolon(content, define_art.end as usize),
        ));
    }
    removal_ranges.sort_unstable();

    let mut setup_bindings = parsed
        .bindings
        .iter()
        .map(|(name, _)| String::from(name))
        .filter(|name| is_valid_identifier(name) && !is_internal_binding(name))
        .collect::<Vec<_>>();
    setup_bindings.sort();
    setup_bindings.dedup();

    let component_bindings = setup_bindings
        .iter()
        .filter(|name| is_component_binding(name))
        .cloned()
        .collect();

    ScriptSetupCsf {
        imports: extract_ranges(content, &import_ranges),
        setup_body: strip_ranges(content, &removal_ranges),
        setup_bindings,
        component_bindings,
    }
}

fn extract_ranges(content: &str, ranges: &[(usize, usize)]) -> String {
    let mut output = String::default();

    for &(start, end) in ranges {
        let Some(import) = content.get(start..end).map(str::trim) else {
            continue;
        };
        if import.is_empty() {
            continue;
        }
        output.push_str(import);
        output.push('\n');
    }

    output
}

fn strip_ranges(content: &str, ranges: &[(usize, usize)]) -> String {
    let mut output = String::default();
    let mut cursor = 0;

    for &(start, end) in ranges {
        if start < cursor || start > content.len() || end > content.len() || start > end {
            continue;
        }
        output.push_str(&content[cursor..start]);
        preserve_newlines(&mut output, &content[start..end]);
        cursor = end;
    }

    output.push_str(&content[cursor..]);
    output.trim().into()
}

fn preserve_newlines(output: &mut String, removed: &str) {
    for ch in removed.chars().filter(|ch| *ch == '\n') {
        output.push(ch);
    }
}

fn extend_through_semicolon(content: &str, mut end: usize) -> usize {
    let bytes = content.as_bytes();
    while matches!(bytes.get(end), Some(b' ' | b'\t' | b'\r')) {
        end += 1;
    }
    if matches!(bytes.get(end), Some(b';')) {
        end += 1;
    }
    end
}

fn is_internal_binding(name: &str) -> bool {
    matches!(name, "args" | "__museaComponent" | "__museaMeta")
}

fn is_component_binding(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase) && is_valid_identifier(name)
}

fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}
