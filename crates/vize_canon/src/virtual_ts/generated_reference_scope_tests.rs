//! Every generated name a projection reads resolves to one of its
//! declarations in an enclosing block.
//!
//! A generated alias or inference-only `const` declared inside one guard block
//! and read from outside it is TS2304/TS2552 on an unmapped generated line.
//! The content mapper drops that diagnostic, and the reading type silently
//! becomes an error type, so the regression only shows as a lost payload.
//!
//! The scan is lexical: braces open blocks, and string, template and comment
//! text is skipped. It checks the names that are generated per module, per
//! usage or per scope (`__vize_*`, or a `__` name carrying an index such as
//! `__Child_11__library`). Helper type parameters and `infer` bindings
//! (`__U`, `__Ctx`, `__P`) carry neither and may shadow a declared name.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use vize_carton::{String, cstr};

/// One generated identifier and the braces open around it.
struct Occurrence<'a> {
    name: &'a str,
    offset: usize,
    blocks: Vec<usize>,
    /// A `var` is hoisted to its function, so no block bounds it.
    function_scoped: bool,
}

#[derive(Clone, Copy)]
enum Open {
    Brace(usize),
    TemplateSubstitution,
}

const DECLARATION_KEYWORDS: [&str; 8] = [
    "const",
    "let",
    "var",
    "type",
    "function",
    "interface",
    "class",
    "enum",
];

fn is_ident(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric()
}

fn is_generated_name(name: &str) -> bool {
    name.starts_with("__vize_")
        || (name.starts_with("__") && name.bytes().any(|byte| byte.is_ascii_digit()))
}

fn previous_significant(code: &[u8], end: usize) -> Option<(usize, u8)> {
    code[..end]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, byte)| !byte.is_ascii_whitespace())
        .map(|(index, byte)| (index, *byte))
}

fn previous_word(code: &str, end: usize) -> &str {
    let bytes = code.as_bytes();
    let Some((last, byte)) = previous_significant(bytes, end) else {
        return "";
    };
    if !is_ident(byte) {
        return "";
    }
    let start = bytes[..=last]
        .iter()
        .rposition(|byte| !is_ident(*byte))
        .map_or(0, |index| index + 1);
    &code[start..=last]
}

fn next_significant(code: &[u8], start: usize) -> &[u8] {
    let rest = &code[start..];
    let skip = rest
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(rest.len());
    &rest[skip..]
}

/// Generated declarations and reads, in source order. String, template and
/// comment text is skipped; property keys and member names are not reads.
fn scan(code: &str) -> (Vec<Occurrence<'_>>, Vec<Occurrence<'_>>) {
    let bytes = code.as_bytes();
    let mut open: Vec<Open> = Vec::new();
    let mut declarations = Vec::new();
    let mut references = Vec::new();
    let mut in_template = false;
    let mut index = 0;
    let blocks = |open: &[Open]| -> Vec<usize> {
        open.iter()
            .filter_map(|open| match open {
                Open::Brace(offset) => Some(*offset),
                Open::TemplateSubstitution => None,
            })
            .collect()
    };
    while index < bytes.len() {
        let byte = bytes[index];
        if in_template {
            match byte {
                b'\\' => index += 2,
                b'`' => {
                    in_template = false;
                    index += 1;
                }
                b'$' if bytes.get(index + 1) == Some(&b'{') => {
                    open.push(Open::TemplateSubstitution);
                    in_template = false;
                    index += 2;
                }
                _ => index += 1,
            }
            continue;
        }
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = bytes[index..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |end| index + end);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = code[index + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |end| index + 2 + end + 2);
            }
            b'\'' | b'"' => {
                index += 1;
                while index < bytes.len() && bytes[index] != byte && bytes[index] != b'\n' {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
                index += 1;
            }
            b'`' => {
                in_template = true;
                index += 1;
            }
            b'{' => {
                open.push(Open::Brace(index));
                index += 1;
            }
            b'}' => {
                if let Some(Open::TemplateSubstitution) = open.pop() {
                    in_template = true;
                }
                index += 1;
            }
            byte if is_ident(byte) => {
                let start = index;
                while index < bytes.len() && is_ident(bytes[index]) {
                    index += 1;
                }
                let name = &code[start..index];
                if !is_generated_name(name) {
                    continue;
                }
                let previous = previous_significant(bytes, start);
                let member = previous
                    .is_some_and(|(at, byte)| byte == b'.' && !code[..=at].ends_with("..."));
                let next = next_significant(bytes, index);
                let annotated = next.starts_with(b":") || next.starts_with(b"?:");
                let after = previous.map(|(_, byte)| byte);
                let keyword = previous_word(code, start);
                let key = keyword == "readonly"
                    || (annotated && matches!(after, None | Some(b'{' | b';')));
                if member || key {
                    continue;
                }
                // `(name: T`, `, name: T` and `...name: T` declare a parameter.
                // It is approximated as declared in the enclosing block.
                let parameter = annotated && matches!(after, Some(b'(' | b',' | b'.'));
                let occurrence = Occurrence {
                    name,
                    offset: start,
                    blocks: blocks(&open),
                    function_scoped: keyword == "var",
                };
                if parameter || DECLARATION_KEYWORDS.contains(&keyword) {
                    declarations.push(occurrence);
                } else {
                    references.push(occurrence);
                }
            }
            _ => index += 1,
        }
    }
    (declarations, references)
}

/// Generated names read where none of their declarations is in scope, with
/// the offset of each read. Names the projection never declares are not
/// checked, nor are `var` names, which their function hoists.
pub(in crate::virtual_ts) fn out_of_scope_references(code: &str) -> Vec<(&str, usize)> {
    let (declarations, references) = scan(code);
    references
        .iter()
        .filter(|reference| {
            let mut declared = declarations
                .iter()
                .filter(|declaration| declaration.name == reference.name)
                .peekable();
            declared.peek().is_some()
                && !declared.any(|declaration| {
                    declaration.function_scoped || reference.blocks.starts_with(&declaration.blocks)
                })
        })
        .map(|reference| (reference.name, reference.offset))
        .collect()
}

/// Panics with the generated line of every out-of-scope read.
pub(in crate::virtual_ts) fn assert_generated_references_in_scope(code: &str) {
    let outside = out_of_scope_references(code);
    if outside.is_empty() {
        return;
    }
    let rows: Vec<String> = outside
        .iter()
        .map(|(name, offset)| {
            let line = code[..*offset].matches('\n').count() + 1;
            let text = code.lines().nth(line - 1).unwrap_or_default().trim();
            cstr!("{name} at line {line}: {text}")
        })
        .collect();
    panic!(
        "generated names read outside their declaration blocks:\n{}\n\n{code}",
        rows.join("\n")
    );
}

fn collect_vue_fixtures(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_vue_fixtures(&path, files);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".vue.txt"))
        {
            files.push(path);
        }
    }
}

/// Cheap guard for the whole family: every typechecker parity fixture's
/// projection reads its generated names from inside their declaring blocks.
#[test]
fn typechecker_fixture_projections_read_generated_names_in_scope() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker");
    let mut files = Vec::new();
    collect_vue_fixtures(&root, &mut files);
    files.sort();
    let mut checked = 0;
    let mut outside = Vec::new();
    for path in &files {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let result = crate::type_check_sfc(
            &source,
            &crate::SfcTypeCheckOptions::new("Probe.vue").with_virtual_ts(),
        );
        let Some(code) = result.virtual_ts else {
            continue;
        };
        checked += 1;
        for (name, offset) in out_of_scope_references(&code) {
            let line = code[..offset].matches('\n').count() + 1;
            outside.push(cstr!(
                "{}: {name} at generated line {line}",
                path.strip_prefix(&root).unwrap_or(path).display()
            ));
        }
    }
    assert!(
        checked > 100,
        "only {checked} fixture projections were checked"
    );
    assert_eq!(
        outside,
        Vec::<String>::new(),
        "generated names read outside their declaration blocks"
    );
}

#[test]
fn a_read_outside_the_declaring_block_is_reported() {
    let code = "{\n  if (a) {\n    const __vize_x_0 = 1;\n    type __A = typeof __vize_x_0;\n  }\n  type __B = typeof __vize_x_0;\n}\n";
    assert_eq!(
        out_of_scope_references(code),
        vec![("__vize_x_0", code.rfind("__vize_x_0").unwrap())]
    );
}

#[test]
fn a_redeclaration_in_the_reading_block_resolves_the_read() {
    let code = "{\n  if (a) {\n    const __vize_x_0 = 1;\n    type __A = typeof __vize_x_0;\n  }\n  const __vize_x_0 = 2;\n  type __B = typeof __vize_x_0 extends { __vize_x_0?: infer __X } ? __X : `${'}'}`;\n}\n";
    assert_eq!(out_of_scope_references(code), Vec::new());
}
