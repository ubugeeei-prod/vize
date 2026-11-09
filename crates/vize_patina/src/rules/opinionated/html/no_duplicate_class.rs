//! html/no-duplicate-class
//!
//! Disallow duplicate statically spelled class names across `class` and
//! dynamic `:class` bindings.
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

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::opinionated::class_values::possible_static_parts;
use oxc_span::Span;
use vize_carton::String;
use vize_l0::{FxHashMap, FxHashSet};
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
    fn report_duplicates(
        ctx: &mut LintContext<'_>,
        candidates: impl IntoIterator<Item = (String, crate::ir::ByteRange)>,
        mut seen: FxHashSet<String>,
    ) {
        let mut reported = FxHashSet::default();
        for (class_name, range) in candidates {
            if !seen.insert(class_name.clone()) && reported.insert(class_name.clone()) {
                let message =
                    ctx.t_fmt("html/no-duplicate-class.message", &[("name", &class_name)]);
                let help = ctx.t("html/no-duplicate-class.help");
                ctx.warn_at_with_help(message, range, help);
            }
        }
    }

    fn report_dynamic_duplicates(
        ctx: &mut LintContext<'_>,
        candidates: impl IntoIterator<Item = (String, crate::ir::ByteRange, bool, Option<Span>)>,
        static_classes: &FxHashSet<String>,
    ) {
        let mut collected = FxHashMap::<String, (bool, Option<Span>)>::default();
        let mut reported = FxHashSet::default();
        let mut current_part = None;
        let mut within_part = FxHashSet::default();
        for (class_name, range, unconditional, shared_group) in candidates {
            let part = (range, unconditional, shared_group);
            if current_part != Some(part) {
                current_part = Some(part);
                within_part.clear();
            }
            let duplicate_within_part = !within_part.insert(class_name.clone());
            let duplicate_with_static = static_classes.contains(&class_name);
            let duplicate_across_parts = collected.get(&class_name).is_some_and(
                |(existing_unconditional, existing_group)| {
                    unconditional
                        || *existing_unconditional
                        || shared_group.is_some_and(|group| Some(group) == *existing_group)
                },
            );
            collected
                .entry(class_name.clone())
                .or_insert((unconditional, shared_group));
            if (duplicate_within_part || duplicate_with_static || duplicate_across_parts)
                && reported.insert(class_name.clone())
            {
                let message =
                    ctx.t_fmt("html/no-duplicate-class.message", &[("name", &class_name)]);
                let help = ctx.t("html/no-duplicate-class.help");
                ctx.warn_at_with_help(message, range, help);
            }
        }
    }

    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        inspect_dynamic: bool,
    ) {
        let mut static_baseline = None;
        let mut static_groups = Vec::new();
        let mut dynamic_groups = Vec::new();
        element.walk_bindings(&mut |binding| {
            if !binding.is_unqualified_arg_exact("class") {
                return;
            }
            match binding.kind() {
                MarkupBindingKind::Attribute => {
                    if let Some(value) = binding.static_value() {
                        let candidates = value
                            .split_ascii_whitespace()
                            .map(|name| (String::from(name), binding.range()))
                            .collect::<Vec<_>>();
                        if static_baseline.is_none() {
                            static_baseline = Some(
                                candidates
                                    .iter()
                                    .map(|(name, _)| name.clone())
                                    .collect::<FxHashSet<_>>(),
                            );
                        }
                        static_groups.push(candidates);
                    }
                }
                MarkupBindingKind::Bind if inspect_dynamic => {
                    let Some(expression) = binding.retained_expression() else {
                        return;
                    };
                    let mut parts = Vec::new();
                    possible_static_parts(expression.ast, &mut parts);
                    let mut candidates = Vec::new();
                    for part in parts {
                        let range = expression
                            .authored_range(part.span)
                            .unwrap_or(binding.range());
                        candidates.extend(part.text.split_ascii_whitespace().map(|name| {
                            (
                                String::from(name),
                                range,
                                part.unconditional,
                                part.shared_group,
                            )
                        }));
                    }
                    dynamic_groups.push(candidates);
                }
                _ => {}
            }
        });

        for candidates in static_groups {
            Self::report_duplicates(ctx, candidates, FxHashSet::default());
        }
        if inspect_dynamic {
            let static_classes = static_baseline.unwrap_or_default();
            for candidates in dynamic_groups {
                Self::report_dynamic_duplicates(ctx, candidates, &static_classes);
            }
        }
    }
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
mod tests {
    use super::NoDuplicateClass;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDuplicateClass));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_classes() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="btn primary">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="btn">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_dynamic_and_cross_binding_duplicates() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div class="a" :class="active ? 'a' : 'b'" />
<div :class="['c c', { 'd d': active }]" />
<div :class="`e e ${active ? 'f' : ''}`" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 4, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_conditional_object_and_ternary_branches_do_not_collide() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="{ 'same': top, 'same': bottom }" />
<div :class="side ? 'same' : 'same'" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_dynamic_template_prefixes_are_not_complete_classes() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[`badge--${variant}`, `badge--${size}`]" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_duplicate_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="btn btn primary">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_two_distinct_duplicates() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="a a b b">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_invalid_triple_duplicate_reports_once() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="x x x">y</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_extra_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="  btn   primary  ">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
