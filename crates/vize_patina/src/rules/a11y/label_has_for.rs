//! a11y/label-has-for
//!
//! Require `<label>` elements to have a `for` attribute or wrap a form control.
//!
//! Labels must be properly associated with form controls so assistive
//! technologies can correctly identify and announce them.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <label>Name</label>
//! ```
//!
//! ### Valid
//! ```vue
//! <label for="name">Name</label>
//! <label><input type="text" /> Name</label>
//! <label :for="id">Name</label>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupNode, MarkupRule,
    MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::a11y::helpers::{
    canonical_component_name, matches_configured_component, string_literal_value,
};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode};

mod control;

use control::{
    is_for_name, is_native_control_tag, markup_hides_control, markup_is_control,
    relief_component_is_control, relief_hides_control,
};

static META: RuleMeta = RuleMeta {
    name: "a11y/label-has-for",
    description: "Require labels to have associated form controls",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require labels to have associated form controls
pub struct LabelHasFor {
    control_components: Vec<vize_l0::String>,
}

#[expect(
    non_upper_case_globals,
    reason = "the unit-like rule value intentionally shares the rule type name"
)]
/// Default `a11y/label-has-for` rule with no project-local controls.
pub const LabelHasFor: LabelHasFor = LabelHasFor {
    control_components: Vec::new(),
};

impl Default for LabelHasFor {
    fn default() -> Self {
        LabelHasFor
    }
}

impl LabelHasFor {
    pub(crate) fn with_control_components(components: Vec<vize_l0::String>) -> Self {
        Self {
            control_components: components
                .into_iter()
                .map(|name| canonical_component_name(&name))
                .collect(),
        }
    }
}

fn nonempty_expression(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || matches!(value, "null" | "undefined") {
        return false;
    }
    string_literal_value(value).is_none_or(|literal| !literal.is_empty())
}

fn has_for_attribute(element: &ElementNode) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => {
            is_for_name(attr.name)
                && attr
                    .value
                    .as_ref()
                    .is_some_and(|value| !value.content.is_empty())
        }
        // `v-bind="attrs"` can supply `for` at runtime (#8961).
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && !dir.shorthand
                && match &dir.arg {
                    Some(ExpressionNode::Simple(s)) if s.is_static && is_for_name(s.content) => {
                        dir.exp.as_ref().is_some_and(|expression| match expression {
                            ExpressionNode::Simple(simple) => nonempty_expression(simple.content),
                            ExpressionNode::Compound(compound) => !compound.children.is_empty(),
                        })
                    }
                    Some(_) => false,
                    None => true,
                }
        }
    })
}

fn has_nested_form_control(
    children: &[TemplateChildNode],
    control_components: &[vize_l0::String],
) -> bool {
    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                if el.tag_type == ElementType::Slot || relief_hides_control(el) {
                    continue;
                }
                if (el.tag_type == ElementType::Element && is_native_control_tag(el.tag))
                    || relief_component_is_control(el)
                    || matches_configured_component(el.tag, control_components)
                {
                    return true;
                }
                if has_nested_form_control(&el.children, control_components) {
                    return true;
                }
            }
            TemplateChildNode::If(if_node) => {
                for branch in &if_node.branches {
                    if has_nested_form_control(&branch.children, control_components) {
                        return true;
                    }
                }
            }
            TemplateChildNode::For(for_node)
                if has_nested_form_control(&for_node.children, control_components) =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn has_markup_for(element: &MarkupElement<'_>, source: &str, template: bool) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= match binding.kind() {
            MarkupBindingKind::Attribute => {
                (binding.is_unqualified_arg_eq_ignore_ascii_case("for")
                    || binding.is_unqualified_arg_eq_ignore_ascii_case("htmlFor"))
                    && binding
                        .static_value()
                        .is_some_and(|value| !value.is_empty())
            }
            MarkupBindingKind::Bind => {
                // A template's dynamic `:[htmlFor]` names whatever it evaluates to.
                let names = |name| {
                    if template {
                        binding.is_static_unqualified_arg_eq_ignore_ascii_case(name)
                    } else {
                        binding.is_unqualified_arg_eq_ignore_ascii_case(name)
                    }
                };
                if binding.arg_name().is_none() {
                    true
                } else if names("for") || names("htmlFor") {
                    let range = binding.range();
                    let has_authored_value = source
                        .get(range.start as usize..range.end as usize)
                        .is_some_and(|authored| authored.contains('='));
                    has_authored_value && binding.expression().is_some_and(nonempty_expression)
                } else {
                    false
                }
            }
            _ => false,
        };
    });
    found
}

/// JSX conditionals and lists count by what they render, as on the lowered path.
fn has_nested_markup_control(
    element: &MarkupElement<'_>,
    control_components: &[vize_l0::String],
) -> bool {
    let mut found = false;
    element.walk_rendered_children(&mut |child| {
        if let MarkupNode::Element(child) = child {
            if child.kind() == MarkupElementKind::Slot || markup_hides_control(&child) {
                return;
            }
            found = found
                || markup_is_control(&child, control_components)
                || has_nested_markup_control(&child, control_components);
        }
    });
    found
}

impl MarkupRule for LabelHasFor {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.kind() == MarkupElementKind::Component
            || !element.is_unqualified_tag_exact("label")
            || has_markup_for(element, ctx.lint().source, ctx.is_template())
            || has_nested_markup_control(element, &self.control_components)
        {
            return;
        }
        let message = ctx.lint().t("a11y/label-has-for.message");
        let help = ctx.lint().t("a11y/label-has-for.help");
        ctx.lint().warn_at_with_help(message, element.range(), help);
    }
}

impl Rule for LabelHasFor {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        if element.tag != "label" {
            return;
        }

        // Has `for` attribute (static or dynamic)
        if has_for_attribute(element) {
            return;
        }

        // Has nested form control
        if has_nested_form_control(&element.children, &self.control_components) {
            return;
        }

        ctx.warn_with_help(
            ctx.t("a11y/label-has-for.message"),
            &element.loc,
            ctx.t("a11y/label-has-for.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::LabelHasFor;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(LabelHasFor));
        Linter::with_registry(registry)
    }

    #[test]
    fn configured_control_component_counts_as_nested_control() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(LabelHasFor::with_control_components(vec![
            "CInputField".into(),
        ])));
        let linter = Linter::with_registry(registry);
        for source in [
            "<label>Name <CInputField /></label>",
            "<label>Name <c-input-field /></label>",
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, 0,
                "{source}: {:?}",
                result.diagnostics
            );
        }
        let result = linter.lint_template("<label>Name <OtherInput /></label>", "test.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_valid_with_for() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<label for="name">Name</label>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_dynamic_for() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<label :for="id">Name</label>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_wrapping_input() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<label>Name <input type="text" /></label>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_wrapping_select() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<label>Choice <select><option>A</option></select></label>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn accepts_all_native_labelable_controls() {
        let linter = create_linter();
        for tag in ["meter", "progress"] {
            let source = format!("<label>Level <{tag} value=\"0.5\" /></label>");
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(
                result.warning_count, 0,
                "{source}: {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn rejects_empty_for_and_hidden_controls() {
        let result = create_linter().lint_template(
            r#"<label for="">Name</label><label>Name <input aria-hidden="true" /></label>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2, "{:?}", result.diagnostics);

        for source in [
            r#"<label :for="''">Name</label>"#,
            r#"<label :for="null">Name</label>"#,
            r#"<label :for>Name</label>"#,
            r#"<label>Name <span aria-hidden="true"><input /></span></label>"#,
        ] {
            let result = create_linter().lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, 1,
                "{source}: {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn test_invalid_no_for_no_control() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<label>Name</label>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<label>Name <span>Help</span></label>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn jsx_controls_inside_conditionals_and_lists_label() {
        let linter = create_linter();
        for (source, expected) in [
            ("const A = () => <label>{cond && <input />}</label>;", 0),
            (
                "const A = () => <label>{items.map((i) => <input />)}</label>;",
                0,
            ),
            ("const A = () => <label>{cond && <span />}</label>;", 1),
        ] {
            let result = linter.lint_jsx(source, "test.jsx", vize_atelier_jsx::JsxLang::Jsx);
            assert_eq!(result.warning_count, expected, "{source}: {result:?}");
        }
    }

    #[test]
    fn third_party_shapes_follow_vue_rendering() {
        let cases: [(&str, usize); 11] = [
            (r#"<label>Name <input :aria-hidden="hidden" /></label>"#, 1),
            (r#"<label><slot><input /></slot></label>"#, 1),
            (r#"<label>Name <component is="input" /></label>"#, 0),
            (r#"<label>Name <component :is="'input'" /></label>"#, 0),
            (r#"<label FOR="name">Name</label>"#, 0),
            // `<Input>` is a component in an SFC, not a native input.
            (r#"<label>Name <Input /></label>"#, 1),
            // Vue omits a binding whose value is `undefined`.
            (r#"<label :for="undefined">Name</label>"#, 1),
            (r#"<label>Name <input aria-hidden /></label>"#, 1),
            (r#"<label>Name <input :aria-hidden="false" /></label>"#, 0),
            (
                r#"<label>Name <span :aria-hidden="hidden"><input /></span></label>"#,
                1,
            ),
            // A bare `v-bind` can still carry `for` (#8961).
            (r#"<label v-bind="attrs">Name</label>"#, 0),
        ];
        for (source, warnings) in cases {
            let template = create_linter().lint_template(source, "test.vue");
            assert_eq!(
                template.warning_count, warnings,
                "template {source}: {:?}",
                template.diagnostics
            );
            let relief = create_linter().lint_template_relief(source, "test.vue");
            let relief_count = relief
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "a11y/label-has-for")
                .count();
            assert_eq!(
                relief_count, warnings,
                "relief {source}: {:?}",
                relief.diagnostics
            );
            let sfc = format!("<template>{source}</template>");
            let file = create_linter().lint_sfc(&sfc, "App.vue");
            let count = file
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "a11y/label-has-for")
                .count();
            assert_eq!(count, warnings, "sfc {source}: {:?}", file.diagnostics);
        }
    }

    #[test]
    fn dynamic_argument_is_not_a_for_attribute() {
        let result =
            create_linter().lint_template(r#"<label :[htmlFor]="id">Name</label>"#, "test.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }
}
