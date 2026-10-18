//! vue/valid-v-bind
//!
//! Enforce valid `v-bind` directives.
//!
//! `v-bind` must:
//! - Have an attribute name (argument) or be used for object binding
//! - Have an expression (or use Vue 3.4+ same-name shorthand)
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-bind></div>
//! <div :></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div :class="foo"></div>
//! <div v-bind:class="foo"></div>
//! <div v-bind="{ class: foo }"></div>
//! <div :loading></div>  <!-- Vue 3.4+ same-name shorthand for :loading="loading" -->
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-bind",
    description: "Enforce valid `v-bind` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-bind directives
pub struct ValidVBind;

const VALID_MODIFIERS: &[&str] = &["attr", "camel", "prop", "sync"];

impl Rule for ValidVBind {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "bind" {
            return;
        }

        for modifier in directive.modifiers.iter() {
            if !VALID_MODIFIERS.contains(&modifier.content) {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-bind.unsupported_modifier"),
                    &modifier.loc,
                    ctx.t("vue/valid-v-bind.help"),
                );
            }
        }

        let has_arg = directive.arg.is_some();
        let has_exp = directive
            .exp
            .as_ref()
            .map(|e| !is_empty_expression(e))
            .unwrap_or(false);

        // Object syntax: v-bind="{ class: foo }"
        if !has_arg && has_exp {
            // This is valid object syntax
            return;
        }

        // Attribute syntax: :class="foo" or Vue 3.4+ same-name shorthand: :loading
        if has_arg {
            // The parser supplies the expression for Vue 3.4+ same-name
            // shorthand. An authored empty value must not use that exemption.
            if !has_exp {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-bind.missing_expression"),
                    &directive.loc,
                    ctx.t("vue/valid-v-bind.help"),
                );
            }
            return;
        }

        // No argument and no expression
        ctx.error_with_help(
            ctx.t("vue/valid-v-bind.missing_expression"),
            &directive.loc,
            ctx.t("vue/valid-v-bind.help"),
        );
    }
}

/// Check if expression is empty
fn is_empty_expression(exp: &ExpressionNode) -> bool {
    match exp {
        ExpressionNode::Simple(s) => s.content.trim().is_empty(),
        ExpressionNode::Compound(c) => c.children.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVBind;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVBind));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_v_bind() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_long_form() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind:class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_same_name_shorthand() {
        // Vue 3.4+ same-name shorthand: :loading is equivalent to :loading="loading"
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :loading></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_same_name_shorthand_multiple() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :loading :disabled :checked></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_dynamic_argument_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :[name]="value"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_bind_supported_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :id.camel="id" :value.prop="value" :aria-label.attr="label" :title.sync="title"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_v_bind_no_arg_no_exp() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_bind_dynamic_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :[name]></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_v_bind_unsupported_modifier() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo.bar="value"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn parser_distinguishes_same_name_shorthand_from_authored_empty_values() {
        use vize_armature::Parser;
        use vize_l0::Allocator;
        use vize_relief::{ExpressionNode, PropNode, TemplateChildNode, errors::ErrorCode};

        for (source, expression, shorthand, missing_value) in [
            (r#"<div :label></div>"#, "label", true, false),
            (r#"<div :label=""></div>"#, "", false, false),
            (r#"<div :label=''></div>"#, "", false, false),
            (r#"<div :label="  "></div>"#, "  ", false, false),
            (r#"<div :[name]=""></div>"#, "", false, false),
            (r#"<div v-bind=""></div>"#, "", false, false),
            ("<div :label=></div>", "", false, true),
            ("<div :label = ></div>", "", false, true),
            ("<div :label=", "", false, true),
            ("<div :label", "label", true, false),
        ] {
            let allocator = Allocator::new();
            let (root, errors) = Parser::new(&allocator, source).parse();
            assert_eq!(
                errors
                    .iter()
                    .any(|error| error.code == ErrorCode::MissingAttributeValue),
                missing_value,
                "{source}: {errors:?}"
            );
            if source.ends_with("</div>") && !missing_value {
                assert!(errors.is_empty(), "{source}: {errors:?}");
            }
            let Some(TemplateChildNode::Element(element)) = root.children.first() else {
                panic!("expected the authored element: {source}");
            };
            let Some(PropNode::Directive(directive)) = element.props.first() else {
                panic!("expected the authored directive: {source}");
            };
            let Some(ExpressionNode::Simple(value)) = directive.exp.as_ref() else {
                panic!("the parser must retain the expression: {source}");
            };
            assert_eq!(value.content, expression, "{source}");
            assert_eq!(directive.shorthand, shorthand, "{source}");
            assert_eq!(
                source.get(value.loc.span.start as usize..value.loc.span.end as usize),
                Some(expression),
                "{source}"
            );
        }
    }

    #[test]
    fn authored_empty_values_report_the_complete_directive() {
        use crate::diagnostic::Severity;

        let linter = create_linter();
        for attribute in [
            r#":label="""#,
            r#":label="  ""#,
            "v-bind:label=' '",
            r#":label.prop="""#,
            r#".label="""#,
            r#":[name]="""#,
            r#"v-bind="""#,
        ] {
            let source = vize_l0::cstr!("<div {attribute}></div>");
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(result.error_count, 1, "{source}");
            assert_eq!(result.warning_count, 0, "{source}");
            assert_eq!(result.diagnostics.len(), 1, "{source}");
            let diagnostic = result.diagnostics.first().expect("one diagnostic");
            assert_eq!(diagnostic.rule_name, "vue/valid-v-bind");
            assert_eq!(diagnostic.severity, Severity::Error);
            assert_eq!(diagnostic.message, "v-bind requires an expression");
            assert_eq!(diagnostic.start, 5);
            assert_eq!(diagnostic.end as usize, 5 + attribute.len());
            assert_eq!(
                source.get(diagnostic.start as usize..diagnostic.end as usize),
                Some(attribute)
            );
        }
    }
}
