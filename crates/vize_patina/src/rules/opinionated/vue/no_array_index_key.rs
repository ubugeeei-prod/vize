//! vue/no-array-index-key
//!
//! Disallow using the `v-for` index variable directly as the `:key`.
//!
//! The `:key` should be a stable, unique identifier tied to the item's
//! identity. Using the loop index (`v-for="(item, index) in list"` with
//! `:key="index"`) defeats Vue's virtual-DOM reconciliation: when the list is
//! reordered, inserted into, or filtered, the index of an item changes, so Vue
//! reuses the wrong element state. Adapted from `react/no-array-index-key` for
//! Vue templates.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <li v-for="(item, index) in list" :key="index">{{ item }}</li>
//! <li v-for="(value, key, index) in obj" :key="index">{{ value }}</li>
//! ```
//!
//! ### Valid
//! ```vue
//! <li v-for="(item, index) in list" :key="item.id">{{ item }}</li>
//! <li v-for="item in list" :key="item.id">{{ item }}</li>
//! <li v-for="(value, key) in object" :key="key">{{ value }}</li>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-array-index-key",
    description: "Disallow using the v-for index variable directly as the :key",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow using the `v-for` index variable directly as the `:key`.
pub struct NoArrayIndexKey;

impl Rule for NoArrayIndexKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Find the `v-for` index alias and the `:key` expression on this same
        // element. Both must be present for the anti-pattern to apply.
        let mut index_alias: Option<&str> = None;

        for prop in element.props.iter() {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name == "for"
                && let Some(ExpressionNode::Simple(exp)) = &dir.exp
            {
                index_alias = v_for_index_alias(exp.content);
            }
        }

        if let (Some(index), Some((key_exp, key_loc))) = (index_alias, key_binding(element))
            && expression_is_only_identifier(key_exp, index)
        {
            report_index_key(ctx, key_loc);
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if element.tag != "template" || directive.name != "for" {
            return;
        }

        let Some(ExpressionNode::Simple(exp)) = &directive.exp else {
            return;
        };
        let Some(index) = v_for_index_alias(exp.content) else {
            return;
        };

        for child in element.children.iter() {
            let TemplateChildNode::Element(child_element) = child else {
                continue;
            };
            if has_v_for(child_element) {
                continue;
            }
            if let Some((key_exp, key_loc)) = key_binding(child_element)
                && expression_is_only_identifier(key_exp, index)
            {
                report_index_key(ctx, key_loc);
            }
        }
    }
}

fn has_v_for(element: &ElementNode<'_>) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "for"))
}

fn key_binding<'a>(
    element: &'a ElementNode<'a>,
) -> Option<(&'a str, &'a vize_relief::SourceLocation)> {
    for prop in element.props.iter() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name == "bind"
            && let Some(ExpressionNode::Simple(arg)) = &dir.arg
            && arg.is_static
            && arg.content == "key"
            && let Some(ExpressionNode::Simple(exp)) = &dir.exp
        {
            return Some((exp.content, &dir.loc));
        }
    }
    None
}

fn report_index_key(ctx: &mut LintContext<'_>, loc: &vize_relief::SourceLocation) {
    ctx.warn_with_help(
        ctx.t("vue/no-array-index-key.message"),
        loc,
        ctx.t("vue/no-array-index-key.help"),
    );
}

/// Extract the index alias from a `v-for` expression string, if any.
///
/// The positional index alias only exists in the parenthesized tuple form:
///
/// - `(item, index) in list` → `index` (the array index)
/// - `(value, key, index) in obj` → `index` (the object iteration index)
///
/// The third binding is always a positional index. The second binding is
/// ambiguous: Vue uses that position for an array index *or* an object property
/// key. Report it by default so unconventionally named array indices remain
/// covered. Exempt object-key names when the source's type is not available;
/// an array or range source still makes any second alias an index.
fn v_for_index_alias(raw: &str) -> Option<&str> {
    let (alias_part, source) = split_for_parts(raw)?;
    let alias_part = alias_part.trim();

    // Only the parenthesized tuple form exposes a positional index.
    if !(alias_part.starts_with('(') && alias_part.ends_with(')')) {
        return None;
    }

    let inner = alias_part.get(1..alias_part.len() - 1).unwrap_or_default();
    let parts: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let index = match parts.as_slice() {
        [_, _, index] => *index,
        [_, _] if is_known_object_source(source) => return None,
        [_, second] if is_known_array_source(source) || !is_object_key_name(second) => *second,
        _ => return None,
    };

    is_plain_identifier(index).then_some(index)
}

/// Split a `v-for` expression on the ` in ` / ` of ` separator.
fn split_for_parts(content: &str) -> Option<(&str, &str)> {
    let bytes = content.as_bytes();
    if let Some(idx) = find_pattern(bytes, b" in ") {
        Some((content.get(..idx)?, content.get(idx + 4..)?.trim()))
    } else {
        let idx = find_pattern(bytes, b" of ")?;
        Some((content.get(..idx)?, content.get(idx + 4..)?.trim()))
    }
}

/// Literal sources can be classified without guessing from variable names.
fn is_known_object_source(source: &str) -> bool {
    source.starts_with('{') && source.ends_with('}')
}

fn is_known_array_source(source: &str) -> bool {
    (source.starts_with('[') && source.ends_with(']'))
        || source.starts_with("Array.from(")
        || source.starts_with("Array.of(")
        || source.starts_with("new Array(")
        || source.starts_with("Object.keys(")
        || source.starts_with("Object.values(")
        || source.starts_with("Object.entries(")
        || (!source.is_empty() && source.bytes().all(|c| c.is_ascii_digit()))
}

/// Common object-property aliases. With an unresolved source, this convention
/// avoids falsely diagnosing object iteration; a visibly array-valued source
/// still treats its second alias as an index regardless of its name.
fn is_object_key_name(name: &str) -> bool {
    matches!(name, "key" | "type" | "source" | "file")
}

/// Returns true when `expression` is exactly the identifier `name` (after
/// trimming), i.e. `:key="index"` rather than `:key="item.id"` or
/// `:key="`row-${index}`"`. Only a bare reference to the index is reported;
/// composing the index into a larger key string is left alone.
fn expression_is_only_identifier(expression: &str, name: &str) -> bool {
    expression.trim() == name
}

/// Returns true when `s` is a plain JS identifier (no member access, calls,
/// destructuring, etc.).
fn is_plain_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Fast byte pattern search.
fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests;
