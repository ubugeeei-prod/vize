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
/// The index is always the *last* binding of a 2- or 3-element tuple. A single
/// alias (`item in list`), object destructuring (`{ id, name } in list`), and
/// array destructuring (`[a, b] in list`) carry no positional index, so this
/// returns `None` for them — their bindings are value properties, not indices.
fn v_for_index_alias(raw: &str) -> Option<&str> {
    let alias_part = split_for_alias(raw)?.trim();

    // Only the parenthesized tuple form exposes a positional index.
    if !(alias_part.starts_with('(') && alias_part.ends_with(')')) {
        return None;
    }

    let inner = &alias_part[1..alias_part.len() - 1];
    let parts: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    // `(value, index)` or `(value, key, index)` — the last binding is the index.
    // A lone `(item)` has no index; more than 3 parts is malformed.
    match parts.len() {
        2 | 3 => {
            let index = *parts.last()?;
            // A destructured index binding (unusual) is not a bare identifier.
            if is_plain_identifier(index) {
                Some(index)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split a `v-for` expression on the ` in ` / ` of ` separator and return the
/// alias part (left of the separator).
fn split_for_alias(content: &str) -> Option<&str> {
    let bytes = content.as_bytes();
    if let Some(idx) = find_pattern(bytes, b" in ") {
        Some(&content[..idx])
    } else {
        find_pattern(bytes, b" of ").map(|idx| &content[..idx])
    }
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
