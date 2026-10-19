//! a11y/interactive-supports-focus
//!
//! Require elements with interactive roles to be focusable.
//!
//! Elements that have interactive ARIA roles (button, link, etc.) must
//! be focusable either natively or via tabindex. In the APG combobox pattern,
//! focus stays on the element with `aria-activedescendant`, so the listbox it
//! controls (`aria-controls`, matched against the listbox `id` as static text
//! or as the same bound expression) and the options inside it are exempt.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div role="button" @click="handle">Click</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div role="button" tabindex="0" @click="handle">Click</div>
//! <button @click="handle">Click</button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::TemplateSyntax;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupDocument, MarkupElement, MarkupNode, MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::FxHashSet;
use vize_relief::RootNode;

use super::{helpers::is_interactive_role, markup_helpers};

static META: RuleMeta = RuleMeta {
    name: "a11y/interactive-supports-focus",
    description: "Require interactive role elements to be focusable",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require interactive role elements to be focusable
#[derive(Default)]
pub struct InteractiveSupportsFocus;

impl InteractiveSupportsFocus {
    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        exempt: &FxHashSet<u32>,
    ) {
        if element.is_component() {
            return;
        }

        // Skip natively interactive elements - they're already focusable
        if markup_helpers::is_interactive_markup_element(element) {
            return;
        }

        // Check if element has an interactive role
        let role = match markup_helpers::get_static_markup_attribute_value(element, "role") {
            Some(r) => r,
            None => return,
        };

        if !is_interactive_role(role) {
            return;
        }

        // APG combobox: focus stays on the combobox, which points at the active
        // option with `aria-activedescendant`, so its listbox and options need
        // no focus of their own (#5941, #9750).
        if matches!(role, "listbox" | "option") && exempt.contains(&element.range().start) {
            return;
        }

        // Element has interactive role but is not natively interactive
        // Check if it's focusable
        let mut bound_tabindex = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Bind
                && binding.is_static_unqualified_arg_exact("tabindex")
            {
                bound_tabindex = true;
            }
        });
        if !markup_helpers::is_focusable_markup_element(element) && !bound_tabindex {
            ctx.warn_at_with_help(
                ctx.t_fmt("a11y/interactive-supports-focus.message", &[("role", role)]),
                element.range(),
                ctx.t("a11y/interactive-supports-focus.help"),
            );
        }
    }

    fn check_document(ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        let exempt = Self::activedescendant_listboxes(document);
        document.walk_elements(&mut |element| {
            Self::check_element(ctx, &element, &exempt);
        });
    }

    /// Start offsets of the listboxes an `aria-activedescendant` owner
    /// controls, and of every element inside them.
    fn activedescendant_listboxes(document: &MarkupDocument<'_>) -> FxHashSet<u32> {
        let mut controlled = FxHashSet::default();
        document.walk_elements(&mut |element| {
            if markup_helpers::has_named_markup_attribute_or_bind(&element, "aria-activedescendant")
            {
                controlled.extend(id_references(&element, "aria-controls"));
            }
        });
        let mut listboxes = FxHashSet::default();
        if controlled.is_empty() {
            return listboxes;
        }
        document.walk_elements(&mut |element| {
            if markup_helpers::get_static_markup_attribute_value(&element, "role")
                == Some("listbox")
                && id_references(&element, "id")
                    .into_iter()
                    .any(|id| controlled.contains(&id))
            {
                // The listbox and every element inside it.
                collect_subtree(&element, &mut listboxes);
            }
        });
        listboxes
    }
}

fn collect_subtree(element: &MarkupElement<'_>, starts: &mut FxHashSet<u32>) {
    starts.insert(element.range().start);
    element.walk_children(&mut |child| {
        if let MarkupNode::Element(child) = child {
            collect_subtree(&child, starts);
        }
    });
}

/// An id an attribute names: a static token, or a bound expression whose
/// source text identifies the value (`:id="listId"` / `:aria-controls="listId"`).
#[derive(PartialEq, Eq, Hash)]
enum IdReference<'a> {
    Static(&'a str),
    Bound(&'a str),
}

/// The ids `name` references; `aria-controls` may list several.
fn id_references<'a>(element: &MarkupElement<'a>, name: &str) -> Vec<IdReference<'a>> {
    let mut ids = Vec::new();
    element.walk_bindings(&mut |binding| {
        if !binding.is_unqualified_arg_exact(name) {
            return;
        }
        let value = match binding.kind() {
            MarkupBindingKind::Attribute => binding.static_value(),
            MarkupBindingKind::Bind => match binding.expression() {
                Some(expression) => match super::helpers::string_literal_value(expression) {
                    Some(literal) => Some(literal),
                    None => {
                        let expression = expression.trim();
                        if !expression.is_empty() {
                            ids.push(IdReference::Bound(expression));
                        }
                        None
                    }
                },
                None => None,
            },
            _ => None,
        };
        if let Some(value) = value {
            ids.extend(value.split_ascii_whitespace().map(IdReference::Static));
        }
    });
    ids
}

impl MarkupRule for InteractiveSupportsFocus {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        if !document.is_template() {
            return;
        }
        Self::check_document(ctx.lint(), document);
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ctx.is_jsx() {
            Self::check_element(ctx.lint(), element, &FxHashSet::default());
        }
    }
}

impl Rule for InteractiveSupportsFocus {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let document = MarkupDocument::new(root, TemplateSyntax::Vue);
        Self::check_document(ctx, &document);
    }
}

#[cfg(test)]
mod tests {
    use super::InteractiveSupportsFocus;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(InteractiveSupportsFocus));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_native_button() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="handle">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_div_role_button_with_tabindex() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" tabindex="0" @click="handle">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_tabindex_minus_one_is_focusable() {
        let linter = create_linter();
        for source in [
            r#"<div role="button" tabindex="-1" @click="handle">Click</div>"#,
            r#"<span role="link" tabindex="-1">Link</span>"#,
        ] {
            let template = linter.lint_template(source, "test.vue");
            assert_eq!(template.warning_count, 0, "{source}");
            let sfc = linter.lint_sfc(
                &format!("<template>\n  {source}\n</template>\n"),
                "Widget.vue",
            );
            assert_eq!(sfc.warning_count, 0, "{source}");
        }
    }

    #[test]
    fn test_invalid_tabindex_below_minus_one() {
        let linter = create_linter();
        let source = r#"<div role="button" tabindex="-2"></div>"#;
        let template = linter.lint_template(source, "test.vue");
        assert_eq!(template.warning_count, 1);
        let sfc = linter.lint_sfc(
            &format!("<template>\n  {source}\n</template>\n"),
            "Widget.vue",
        );
        assert_eq!(sfc.warning_count, 1);
    }

    #[test]
    fn test_valid_bound_tabindex() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="tab" :tabindex="disabled ? -1 : 0"></div><div role="tab" v-bind:tabindex="0"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_div_role_button_no_tabindex() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" @click="handle">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_non_interactive_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="presentation">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_span_role_link() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span role="link">Link</span>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_activedescendant_listbox() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div>
  <button
    type="button"
    role="combobox"
    aria-controls="lb"
    :aria-activedescendant="`lb-option-${focused}`"
  >Select</button>
  <div id="lb" role="listbox">
    <button role="option" tabindex="-1">A</button>
  </div>
</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn combobox_listbox_and_options_need_no_focus() {
        let linter = create_linter();
        for ids in [
            r#"aria-controls="fruits" | id="fruits""#,
            r#":aria-controls="listId" | :id="listId""#,
            r#":aria-controls="'fruits'" | id="fruits""#,
            r#"aria-controls="other fruits" | :id="`fruits`""#,
        ] {
            let (controls, id) = ids.split_once(" | ").unwrap_or_default();
            let source = format!(
                r#"<input aria-activedescendant="fruit-0" {controls} aria-expanded="true" aria-label="Fruit" role="combobox" />
<ul {id} role="listbox">
  <li id="fruit-0" aria-selected="true" role="option">Apple</li>
  <li role="option"><span>Banana</span></li>
</ul>"#
            );
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.warning_count, 0, "{ids}: {:?}", result.diagnostics);
        }
    }

    #[test]
    fn combobox_with_bound_ids_in_an_sfc() {
        let source = r#"<script setup lang="ts">
const listId = "fruits";
</script>

<template>
  <input
    aria-activedescendant="fruit-0"
    :aria-controls="listId"
    aria-expanded="true"
    aria-label="Fruit"
    role="combobox"
  />
  <ul :id="listId" role="listbox">
    <li id="fruit-0" aria-selected="true" role="option">Apple</li>
  </ul>
</template>
"#;
        let result = create_linter().lint_sfc(source, "BoundIds.vue");
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn listboxes_without_an_activedescendant_owner_still_need_focus() {
        let linter = create_linter();
        for source in [
            // No aria-activedescendant on the controlling element.
            r#"<input :aria-controls="listId" role="combobox" /><ul :id="listId" role="listbox"><li role="option">A</li></ul>"#,
            // A different expression is a different id.
            r#"<input aria-activedescendant="a" :aria-controls="listId" role="combobox" /><ul :id="otherId" role="listbox"><li role="option">A</li></ul>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, 2,
                "{source}: {:?}",
                result.diagnostics
            );
        }
        // Options outside the controlled listbox are still checked.
        let outside = r#"<input aria-activedescendant="a" aria-controls="lb" role="combobox" /><ul id="lb" role="listbox"><li role="option">A</li></ul><div role="option">B</div>"#;
        assert_eq!(linter.lint_template(outside, "test.vue").warning_count, 1);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyButton role="button">Click</MyButton>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
