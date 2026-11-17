//! html/id-duplication
//!
//! Detect duplicate static `id` attribute values in the same template.
//! Based on markuplint's `id-duplication` rule.
//!
//! This is different from `vue/use-unique-element-ids` which warns about
//! any static ID. This rule only warns when the same ID literal appears
//! more than once.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div id="content">first</div>
//!   <div id="content">second</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div id="content">first</div>
//!   <div id="sidebar">second</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::ir::TemplateSyntax;
use crate::markup::{
    MarkupBindingKind, MarkupBranch, MarkupContext, MarkupDocument, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::FxHashMap;
use vize_l0::String;
use vize_l0::ToCompactString;
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "html/id-duplication",
    description: "Disallow duplicate element IDs",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
/// Implements the `html/id-duplication` lint rule.
pub struct IdDuplication;

struct IdEntry {
    value: String,
    start: u32,
    end: u32,
    branches: Vec<MarkupBranch>,
}

impl IdDuplication {
    fn check_document(ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        let ids = collect_static_ids(document);

        // Find duplicates
        let mut seen: FxHashMap<&str, Vec<&IdEntry>> = FxHashMap::default();

        for entry in &ids {
            let duplicate = seen.get(entry.value.as_str()).and_then(|entries| {
                entries
                    .iter()
                    .find(|prev| MarkupBranch::can_coexist(&prev.branches, &entry.branches))
            });

            if let Some(first) = duplicate {
                let message = ctx.t_fmt(
                    "html/id-duplication.message",
                    &[("id", entry.value.as_str())],
                );
                let help = ctx.t("html/id-duplication.help");
                let diag = LintDiagnostic::error(META.name, message, entry.start, entry.end)
                    .with_help(help.into_owned())
                    .with_label(
                        "first defined here".to_compact_string(),
                        first.start,
                        first.end,
                    );
                ctx.report(diag);
            }
            seen.entry(entry.value.as_str()).or_default().push(entry);
        }
    }
}

/// Every valued static `id`, with the alternatives its element renders in.
fn collect_static_ids(document: &MarkupDocument<'_>) -> Vec<IdEntry> {
    let mut ids = Vec::new();
    document.walk_branched_elements(&mut |element, branches| {
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact("id")
                && let Some(value) = binding.static_value()
            {
                let range = binding.range();
                ids.push(IdEntry {
                    value: value.to_compact_string(),
                    start: range.start,
                    end: range.end,
                    branches: branches.to_vec(),
                });
            }
        });
    });
    ids
}

impl MarkupRule for IdDuplication {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Document
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        Self::check_document(ctx.lint(), document);
    }
}

impl Rule for IdDuplication {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // Ids are compared within each render root, across the branches its
        // conditionals choose between.
        true
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        Self::check_document(ctx, &MarkupDocument::new(root, TemplateSyntax::Vue));
    }
}

#[cfg(test)]
mod tests {
    use super::IdDuplication;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(IdDuplication));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_ids() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div id="a">A</div><div id="b">B</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_ids() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>A</div><div>B</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_dynamic_ids() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :id="id1">A</div><div :id="id2">B</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_ids() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="content">A</div><div id="content">B</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_triple_duplicate() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="x">A</div><div id="x">B</div><div id="x">C</div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_nested_duplicate() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="foo"><span id="foo">text</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_duplicate_ids_across_v_if_branches() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show"><input id="radio1"></div><div v-else><input id="radio1"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_ids_inside_same_v_if_branch() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show"><input id="radio1"><p id="radio1"></p></div><div v-else><input id="radio1"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_id_after_v_if_branch() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show"><input id="radio1"></div><div v-else><input id="other"></div><p id="radio1"></p>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1);
    }
}
