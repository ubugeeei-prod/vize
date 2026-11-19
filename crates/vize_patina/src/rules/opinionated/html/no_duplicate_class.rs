//! html/no-duplicate-class
//!
//! Disallow duplicate statically spelled class names across `class` and
//! dynamic `:class` bindings.
//!
//! Findings are reported where `vue/no-duplicate-class-names` reports them,
//! one finding per node listing every duplicated name:
//! - a static `class` value with repeated names, on the value;
//! - a string, object key or template quasi of a `:class` binding with
//!   repeated names, on that node;
//! - a `:class` name the static `class` also lists, on the start tag;
//! - a name two `:class` parts can both render, on the node that holds the
//!   first part (the binding value, array, conditional, property or
//!   template literal).
//!
//! A template quasi contributes only the names bounded by whitespace or the
//! literal's ends, so `badge--` in `` `badge--${variant}` `` is not a class.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div class="btn btn primary">click</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div class="btn primary">click</div>
//! </template>
//! ```

mod report_nodes;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{FxHashSet, String};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "html/no-duplicate-class",
    description: "Disallow duplicate class names in a static class attribute",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoDuplicateClass;

impl NoDuplicateClass {
    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        inspect_dynamic: bool,
    ) {
        let vue = ctx.filename.ends_with(".vue");
        let mut static_classes = None::<&str>;
        element.walk_bindings(&mut |binding| {
            if binding.kind() != MarkupBindingKind::Attribute
                || !binding.is_unqualified_arg_exact("class")
            {
                return;
            }
            let Some(value) = binding.static_value() else {
                return;
            };
            // A Vue file reports once on the value, as eslint-plugin-vue does;
            // JSX and HTML keep one finding per name on the whole attribute.
            if vue {
                let range = report_nodes::attribute_value_range(ctx.source, binding.range())
                    .unwrap_or(binding.range());
                report(ctx, range, &repeated_names(value));
            } else {
                for name in repeated_names(value) {
                    report(ctx, binding.range(), &[name]);
                }
            }
            static_classes.get_or_insert(value);
        });
        if !inspect_dynamic {
            return;
        }
        let static_classes = static_classes.unwrap_or_default();
        let source = ctx.source;
        let mut open_tag = None;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Bind
                && binding.is_unqualified_arg_exact("class")
            {
                let open_tag = *open_tag.get_or_insert_with(|| element.open_tag_range(source));
                report_nodes::check_binding(ctx, &binding, open_tag, static_classes);
            }
        });
    }
}

/// The names that occur more than once, in the order they repeat. A value
/// with fewer than two names allocates nothing.
fn repeated_names(text: &str) -> Vec<&str> {
    let mut repeated = Vec::new();
    let mut names = text.split_ascii_whitespace();
    let (Some(first), Some(second)) = (names.next(), names.next()) else {
        return repeated;
    };
    let mut seen = FxHashSet::default();
    seen.insert(first);
    for name in std::iter::once(second).chain(names) {
        if !seen.insert(name) && !repeated.contains(&name) {
            repeated.push(name);
        }
    }
    repeated
}

/// One finding listing `names`; nothing when the list is empty.
fn report(ctx: &mut LintContext<'_>, range: ByteRange, names: &[&str]) {
    let message = match names {
        [] => return,
        [name] => ctx.t_fmt("html/no-duplicate-class.message", &[("name", name)]),
        names => {
            let mut quoted = String::default();
            for (index, name) in names.iter().enumerate() {
                if index > 0 {
                    quoted.push_str(", ");
                }
                quoted.push('\'');
                quoted.push_str(name);
                quoted.push('\'');
            }
            ctx.t_fmt(
                "html/no-duplicate-class.message_many",
                &[("names", &quoted)],
            )
        }
    };
    let help = ctx.t("html/no-duplicate-class.help");
    ctx.warn_at_with_help(message, range, help);
}

impl MarkupRule for NoDuplicateClass {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let inspect_dynamic = ctx.is_template();
        Self::check_element(ctx.lint(), element, inspect_dynamic);
    }
}

impl Rule for NoDuplicateClass {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element), true);
    }
}

#[cfg(test)]
mod tests;
