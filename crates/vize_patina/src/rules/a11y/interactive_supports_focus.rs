//! a11y/interactive-supports-focus
//!
//! Require elements with interactive roles to be focusable.
//!
//! Elements that have interactive ARIA roles (button, link, etc.) must
//! be focusable either natively or via tabindex. In the APG combobox pattern,
//! focus stays on the element with `aria-activedescendant`, so the listbox it
//! controls (`aria-controls`, matched against the listbox `id` as static text
//! or as the same bound expression) and the options inside it are exempt.
//! A listbox may instead move DOM focus among its owned options. A physical,
//! focusable option establishes that contract without a second container stop.
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
    MarkupBindingKind, MarkupContext, MarkupDocument, MarkupElement, MarkupElementKind, MarkupNode,
    MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::FxHashSet;
use vize_relief::RootNode;

use super::helpers::{INTERACTIVE_HANDLER_EVENTS, is_interactive_role as is_interactive_role_name};
use super::markup_helpers;
use super::reference_values::is_hidden_from_screen_reader;
use super::vuejs_accessibility::{
    AttrValue, INTERACTIVE_HANDLERS, attribute, bound_branches, has_on_directives, is_dom_element,
    is_interactive_element, is_interactive_role, is_jsx_filename, is_presentation_role,
    js_string_to_number,
};

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
        template: bool,
    ) {
        let role = if template {
            Self::unfocusable_template_role(element)
        } else {
            Self::unfocusable_jsx_role(element)
        };
        let Some(role) = role else {
            return;
        };

        // APG combobox: focus stays on the combobox, which points at the active
        // option with `aria-activedescendant`, so its listbox and options need
        // no focus of their own (#5941, #9750).
        if matches!(role, "listbox" | "option") && exempt.contains(&element.range().start) {
            return;
        }

        // APG roving tabindex keeps DOM focus on options, rather than adding a
        // container tab stop. Only an option owned by this listbox can supply it.
        if role == "listbox" && Self::has_focusable_owned_option(element) {
            return;
        }

        ctx.warn_at_with_help(
            ctx.t_fmt("a11y/interactive-supports-focus.message", &[("role", role)]),
            element.range(),
            ctx.t("a11y/interactive-supports-focus.help"),
        );
    }

    /// The plugin's condition, on a DOM element of any authored case.
    fn unfocusable_template_role<'a>(element: &MarkupElement<'a>) -> Option<&'a str> {
        let role_value = attribute(element, "role");
        let role = role_value.and_then(AttrValue::as_str)?;
        (is_interactive_role(role_value)
            && has_on_directives(element, INTERACTIVE_HANDLERS, true)
            && is_dom_element(element)
            && !has_tab_index(element)
            && !is_disabled_element(element)
            && !is_hidden_from_screen_reader(element)
            && !is_presentation_role(element)
            && !is_interactive_element(element))
        .then_some(role)
    }

    /// JSX keeps its exact static attribute and handler lookups.
    fn unfocusable_jsx_role<'a>(element: &MarkupElement<'a>) -> Option<&'a str> {
        if element.is_component() || markup_helpers::is_interactive_markup_element(element) {
            return None;
        }
        let role = markup_helpers::get_static_markup_attribute_value(element, "role")?;
        (is_interactive_role_name(role)
            && markup_helpers::has_static_markup_handler(element, INTERACTIVE_HANDLER_EVENTS)
            && !markup_helpers::is_focusable_markup_element(element)
            && !Self::has_bound_tabindex(element))
        .then_some(role)
    }

    fn has_bound_tabindex(element: &MarkupElement<'_>) -> bool {
        let mut bound_tabindex = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Bind
                && binding.is_static_unqualified_arg_exact("tabindex")
            {
                bound_tabindex = true;
            }
        });
        bound_tabindex
    }

    fn has_focusable_owned_option(element: &MarkupElement<'_>) -> bool {
        let mut found = false;
        element.walk_children(&mut |child| {
            if found {
                return;
            }
            let MarkupNode::Element(child) = child else {
                return;
            };
            // Component/slot placement is unknown. Hidden or inert subtrees
            // cannot provide a DOM focus target for the owning listbox.
            if matches!(
                child.kind(),
                MarkupElementKind::Component | MarkupElementKind::Slot
            ) || child.is_unqualified_tag_exact("slot")
                || markup_helpers::has_static_hidden(&child)
                || markup_helpers::has_static_inert(&child)
            {
                return;
            }
            match markup_helpers::get_static_markup_attribute_value(&child, "role") {
                Some("option") => {
                    found = !markup_helpers::is_statically_non_focusable(&child)
                        && (markup_helpers::is_focusable_markup_element(&child)
                            || Self::has_bound_tabindex(&child));
                }
                // Physical wrappers and owned groups may carry options. A
                // nested listbox or any other widget owns its own descendants.
                None | Some("group" | "presentation" | "none") => {
                    found = Self::has_focusable_owned_option(&child);
                }
                Some(_) => {}
            }
        });
        found
    }

    fn check_document(ctx: &mut LintContext<'_>, document: &MarkupDocument<'_>) {
        let template = !is_jsx_filename(ctx.filename);
        let exempt = Self::activedescendant_listboxes(document);
        document.walk_elements(&mut |element| {
            Self::check_element(ctx, &element, &exempt, template);
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

/// The plugin's `hasTabIndex`. A conditional whose every branch is an
/// integer is a tab index too; the plugin reads it as one opaque value.
fn has_tab_index(element: &MarkupElement<'_>) -> bool {
    let is_integer = |value: AttrValue<'_>| match value {
        AttrValue::Str(value) => !value.is_empty() && is_integer_number(js_string_to_number(value)),
        AttrValue::Num(value) => is_integer_number(value),
        _ => false,
    };
    match attribute(element, "tabindex") {
        None | Some(AttrValue::Bool(_)) => false,
        Some(AttrValue::Null) => true,
        Some(AttrValue::Expr) => {
            let mut all = true;
            bound_branches(element, "tabindex", &mut |value| all &= is_integer(value)) && all
        }
        Some(value) => is_integer(value),
    }
}

fn is_integer_number(value: f64) -> bool {
    value.is_finite() && value.trunc() == value
}

/// The plugin's `isDisabledElement`.
fn is_disabled_element(element: &MarkupElement<'_>) -> bool {
    attribute(element, "disabled").is_some_and(AttrValue::is_truthy)
        || matches!(
            attribute(element, "aria-disabled"),
            Some(AttrValue::Str("true") | AttrValue::Bool(true))
        )
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

    fn traversal(&self) -> crate::markup::MarkupTraversal {
        crate::markup::MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ctx.is_jsx() {
            Self::check_element(ctx.lint(), element, &FxHashSet::default(), false);
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

    pub(super) fn create_legacy_linter() -> Linter {
        struct Legacy;
        impl crate::rule::Rule for Legacy {
            fn meta(&self) -> &'static crate::rule::RuleMeta {
                crate::rule::Rule::meta(&InteractiveSupportsFocus)
            }

            fn run_on_template<'a>(
                &self,
                ctx: &mut crate::context::LintContext<'a>,
                root: &vize_relief::RootNode<'a>,
            ) {
                crate::rule::Rule::run_on_template(&InteractiveSupportsFocus, ctx, root);
            }
        }
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(Legacy));
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
    fn any_integer_tabindex_is_a_tab_index() {
        let linter = create_linter();
        let source = r#"<div role="button" tabindex="-2" @click="handle"></div>"#;
        let template = linter.lint_template(source, "test.vue");
        assert_eq!(template.warning_count, 0);
        let sfc = linter.lint_sfc(
            &format!("<template>\n  {source}\n</template>\n"),
            "Widget.vue",
        );
        assert_eq!(sfc.warning_count, 0);
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
        let result = linter.lint_template(
            r#"<span role="link" @click="follow">Link</span>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyButton role="button">Click</MyButton>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}

#[cfg(test)]
pub(super) fn create_legacy_linter_for_test() -> crate::linter::Linter {
    tests::create_legacy_linter()
}
