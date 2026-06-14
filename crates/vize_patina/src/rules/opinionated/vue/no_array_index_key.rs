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
use vize_relief::{ElementNode, ExpressionNode, PropNode};

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
        let mut key_binding: Option<(&str, &vize_relief::SourceLocation)> = None;

        for prop in element.props.iter() {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            match dir.name.as_str() {
                "for" => {
                    if let Some(ExpressionNode::Simple(exp)) = &dir.exp {
                        index_alias = v_for_index_alias(exp.content.as_str());
                    }
                }
                "bind" => {
                    // `:key` / `v-bind:key` with a dynamic expression.
                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && arg.content.as_str() == "key"
                        && let Some(ExpressionNode::Simple(exp)) = &dir.exp
                    {
                        key_binding = Some((exp.content.as_str(), &dir.loc));
                    }
                }
                _ => {}
            }
        }

        if let (Some(index), Some((key_exp, key_loc))) = (index_alias, key_binding)
            && expression_is_only_identifier(key_exp, index)
        {
            ctx.warn_with_help(
                ctx.t("vue/no-array-index-key.message"),
                key_loc,
                ctx.t("vue/no-array-index-key.help"),
            );
        }
    }
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
mod tests {
    use super::NoArrayIndexKey;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoArrayIndexKey));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_index_used_as_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in list" :key="index">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_object_iteration_index_used_as_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(value, key, index) in obj" :key="index">{{ value }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_index_with_spaces_in_key() {
        let linter = create_linter();
        // `:key=" index "` is still just the index identifier.
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in list" :key=" index ">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_v_bind_key_long_form() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in list" v-bind:key="index">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_stable_id_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in list" :key="item.id">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_index_composed_into_key() {
        let linter = create_linter();
        // Using the index as part of a larger key string is not a bare index.
        let result = linter.lint_template(
            r#"<li v-for="(item, index) in list" :key="`row-${index}`">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_object_3tuple_key_used_as_key() {
        // For object iteration `(value, key, index)`, the *second* binding is
        // the stable object key (not a positional index), so `:key="key"` is
        // fine — only the third binding (`index`) is the positional counter.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(value, key, index) in obj" :key="key">{{ value }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_no_index_alias() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="item in list" :key="item.id">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_index_like_identifier_that_is_not_the_alias() {
        // `idx` is the v-for index; `:key="index"` references some unrelated
        // outer `index`, not the loop index, so it must not be flagged.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, idx) in list" :key="index">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_of_delimiter_with_stable_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) of list" :key="item.id">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_of_delimiter_index_as_key() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="(item, index) of list" :key="index">{{ item }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn ignores_object_destructuring_value_used_as_key() {
        // `{ id }` destructures the value; `id` is not a positional index.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<li v-for="{ id } in list" :key="id">{{ id }}</li>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
