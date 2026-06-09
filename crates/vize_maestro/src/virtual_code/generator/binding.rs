//! Identifier scanning helpers for determining which `<script setup>` bindings
//! are referenced by template expressions.

use std::collections::BTreeSet;

use super::TemplateExpression;
use super::extract_simple_bindings;

pub(super) fn template_used_script_bindings(
    script_content: &str,
    expressions: &[TemplateExpression],
) -> Vec<String> {
    let script_bindings = extract_simple_bindings(script_content, true)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if script_bindings.is_empty() {
        return Vec::new();
    }

    let mut used = BTreeSet::new();
    for expression in expressions {
        collect_expression_identifiers(&expression.text, &script_bindings, &mut used);
    }

    used.into_iter().collect()
}

fn collect_expression_identifiers(
    expression: &str,
    script_bindings: &BTreeSet<String>,
    used: &mut BTreeSet<String>,
) {
    let bytes = expression.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if !is_identifier_start(byte) {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < bytes.len() && is_identifier_continue(bytes[index]) {
            index += 1;
        }

        if is_property_access(expression, start) {
            continue;
        }

        let name = &expression[start..index];
        if !is_js_keyword(name) && script_bindings.contains(name) {
            used.insert(name.to_string());
        }
    }
}

#[inline]
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

#[inline]
fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn is_property_access(expression: &str, start: usize) -> bool {
    expression
        .as_bytes()
        .get(..start)
        .unwrap_or_default()
        .iter()
        .rev()
        .find(|byte| !byte.is_ascii_whitespace())
        .is_some_and(|byte| *byte == b'.')
}

fn is_js_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "from"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "return"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "undefined"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}
