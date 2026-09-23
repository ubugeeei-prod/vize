//! `<style module>` declarations that repeat a module name.
//!
//! Vue merges every `<style module="x">` block into one `x` object at runtime,
//! but Vue Language Tools declares each block as its own property of one type
//! literal, so a repeated name is a duplicate identifier (TS2300 on every
//! occurrence) and a second block with different classes also changes the
//! declared type (TS2717). `vue-tsc` therefore rejects the SFC; this emits the
//! same literal so the same two diagnostics land on the same attributes.
//!
//! The literal is appended after every mapped byte of the module and only
//! when a name repeats, so a file with distinct module names generates
//! exactly what it did before.

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::String;

use super::super::build::css_module_block_type;
use crate::virtual_ts::{VizeMapping, push_ts_string_literal};

pub(super) fn append_duplicate_style_modules(
    code: &mut String,
    mappings: &mut Vec<VizeMapping>,
    source: &str,
    descriptor: &SfcDescriptor,
    strict: bool,
) {
    let modules: Vec<_> = descriptor
        .styles
        .iter()
        .filter_map(|style| {
            let name = style.module.as_deref()?;
            let span = module_attribute_span(source, style.loc.tag_start, style.loc.start)?;
            Some((name, span, css_module_block_type(style, strict)))
        })
        .collect();
    let repeated = modules
        .iter()
        .any(|(name, ..)| modules.iter().filter(|(other, ..)| other == name).count() > 1);
    if !repeated {
        return;
    }
    code.push_str("\ntype __VizeStyleModuleDeclarations = {");
    for (name, span, block_type) in &modules {
        code.push(' ');
        let key_start = code.len();
        push_ts_string_literal(code, name);
        mappings.push(VizeMapping {
            gen_range: key_start..code.len(),
            src_range: span.clone(),
            sub_spans: Vec::new(),
        });
        code.push_str(": ");
        code.push_str(block_type.as_str());
        code.push(';');
    }
    code.push_str(" };\nvoid ({} as __VizeStyleModuleDeclarations);\n");
}

/// The authored bytes `vue-tsc` reports a module declaration at: the value
/// of `module="name"`, or the attribute name itself for a bare `module` and
/// for `module=""`.
fn module_attribute_span(
    source: &str,
    tag_start: usize,
    content_start: usize,
) -> Option<std::ops::Range<usize>> {
    let tag = source.get(tag_start..content_start)?;
    let bytes = tag.as_bytes();
    let mut index = 0;
    while let Some(found) = tag.get(index..).and_then(|rest| rest.find("module")) {
        let start = index + found;
        let end = start + "module".len();
        let preceded = crate::text_scan::byte_before(bytes, start)
            .is_some_and(|byte| byte.is_ascii_whitespace());
        let followed = bytes
            .get(end)
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b'=' | b'/' | b'>'));
        if !(preceded && followed) {
            index = end;
            continue;
        }
        let rest = tag.get(end..).unwrap_or_default();
        let value = rest.strip_prefix('=').and_then(|after| {
            let quote = after.as_bytes().first().copied()?;
            if !matches!(quote, b'"' | b'\'') {
                return None;
            }
            let value_start = end + 2;
            let value_len = after.get(1..)?.find(quote as char)?;
            (value_len > 0).then_some(value_start..value_start + value_len)
        });
        return Some(match value {
            Some(range) => tag_start + range.start..tag_start + range.end,
            None => tag_start + start..tag_start + end,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::module_attribute_span;

    #[test]
    fn module_attribute_spans_follow_vue_language_tools() {
        for (tag, expected) in [
            ("<style module>", Some(7..13)),
            ("<style module=\"\">", Some(7..13)),
            ("<style module=\"foo\">", Some(15..18)),
            ("<style module=\"a-b\" scoped>", Some(15..18)),
            ("<style scoped module='x'>", Some(22..23)),
            ("<style lang=\"scss\">", None),
            ("<style data-module=\"x\">", None),
        ] {
            let source = format!("{tag}.a {{}}</style>");
            assert_eq!(
                module_attribute_span(&source, 0, tag.len()),
                expected,
                "{tag}"
            );
        }
    }
}
