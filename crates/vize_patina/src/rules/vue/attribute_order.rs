//! vue/attribute-order
//!
//! Enforce a consistent order of attributes on elements.
//!
//! Following the Vue.js style guide recommendation, attributes should be
//! ordered as follows:
//!
//! 1. Definition: `is`
//! 2. List Rendering: `v-for`
//! 3. Conditionals: `v-if`, `v-else-if`, `v-else`, `v-show`, `v-cloak`
//! 4. Render Modifiers: `v-pre`, `v-once`
//! 5. Global Awareness: `id`
//! 6. Unique Attributes: `ref`, `key`
//! 7. Two-Way Binding: `v-model`
//! 8. Other Attributes: other bound/unbound attributes
//! 9. Events: `v-on`, `@`
//! 10. Content: `v-html`, `v-text`
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div @click="onClick" v-if="show" id="main"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-if="show" id="main" @click="onClick"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/attribute-order",
    description: "Enforce a consistent order of attributes",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AttrCategory {
    Definition,
    ListRendering,
    Conditionals,
    RenderModifiers,
    GlobalAwareness,
    UniqueAttrs,
    TwoWayBinding,
    OtherDirectives,
    OtherAttrs,
    Events,
    Content,
}

impl AttrCategory {
    fn from_prop(prop: &PropNode) -> Self {
        match prop {
            PropNode::Attribute(attr) => match attr.name.as_str() {
                "is" => AttrCategory::Definition,
                "id" => AttrCategory::GlobalAwareness,
                "ref" | "key" => AttrCategory::UniqueAttrs,
                _ => AttrCategory::OtherAttrs,
            },
            PropNode::Directive(dir) => {
                let arg = dir.arg.as_ref().and_then(|arg| match arg {
                    ExpressionNode::Simple(simple) => Some(simple.content.as_str()),
                    _ => None,
                });

                match dir.name.as_str() {
                    "for" => AttrCategory::ListRendering,
                    "if" | "else-if" | "else" | "show" | "cloak" => AttrCategory::Conditionals,
                    "pre" | "once" => AttrCategory::RenderModifiers,
                    "model" => AttrCategory::TwoWayBinding,
                    "on" => AttrCategory::Events,
                    "html" | "text" => AttrCategory::Content,
                    "bind" => match arg {
                        Some("key") => AttrCategory::UniqueAttrs,
                        Some("is") => AttrCategory::Definition,
                        _ => AttrCategory::OtherAttrs,
                    },
                    "slot" => AttrCategory::OtherDirectives,
                    _ => AttrCategory::OtherDirectives,
                }
            }
        }
    }
}

pub struct AttributeOrder;

impl Rule for AttributeOrder {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.props.len() < 2 {
            return;
        }

        let mut previous_category = None;

        for prop in element.props.iter() {
            let category = AttrCategory::from_prop(prop);

            if let Some(previous_category_value) = previous_category {
                if category < previous_category_value {
                    let loc = match prop {
                        PropNode::Attribute(attr) => &attr.loc,
                        PropNode::Directive(dir) => &dir.loc,
                    };

                    ctx.warn_with_help(
                        ctx.t("vue/attribute-order.message"),
                        loc,
                        ctx.t("vue/attribute-order.help"),
                    );
                }
            }

            previous_category = Some(category);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AttributeOrder;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AttributeOrder));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_order() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-if="show" id="main" ref="el" :class="cls" @click="onClick"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_event_before_conditional() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div @click="onClick" v-if="show"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_v_for_before_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="item in items" :key="item.id"><div v-if="item.visible"></div></template>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_id_before_v_for() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div id="list" v-for="item in items" :key="item.id"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}
