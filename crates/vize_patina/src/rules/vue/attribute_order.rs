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
use vize_relief::{ElementNode, ExpressionNode, PropNode};

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
            PropNode::Attribute(attr) => match attr.name {
                "is" => AttrCategory::Definition,
                "id" => AttrCategory::GlobalAwareness,
                "ref" | "key" | "slot" | "slot-scope" => AttrCategory::UniqueAttrs,
                _ => AttrCategory::OtherAttrs,
            },
            PropNode::Directive(dir) => {
                let arg = dir.arg.as_ref().and_then(|arg| match arg {
                    ExpressionNode::Simple(simple) => Some(simple.content),
                    _ => None,
                });

                match dir.name {
                    "for" => AttrCategory::ListRendering,
                    "if" | "else-if" | "else" | "show" | "cloak" => AttrCategory::Conditionals,
                    "pre" | "once" => AttrCategory::RenderModifiers,
                    "model" => AttrCategory::TwoWayBinding,
                    "on" => AttrCategory::Events,
                    "html" | "text" => AttrCategory::Content,
                    "slot" => AttrCategory::UniqueAttrs,
                    "is" => AttrCategory::Definition,
                    "bind" => match arg {
                        Some("key") => AttrCategory::UniqueAttrs,
                        Some("is") => AttrCategory::Definition,
                        Some("id") => AttrCategory::GlobalAwareness,
                        Some("ref" | "slot" | "slot-scope") => AttrCategory::UniqueAttrs,
                        _ => AttrCategory::OtherAttrs,
                    },
                    _ => AttrCategory::OtherDirectives,
                }
            }
        }
    }
}

fn is_ordering_barrier(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Directive(dir) => dir.name == "bind" && dir.arg.is_none(),
        PropNode::Attribute(attr) => attr
            .value
            .as_ref()
            .is_some_and(|value| value.content.contains("{{")),
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

        let mut highest_category = None;

        for prop in element.props.iter() {
            if is_ordering_barrier(prop) {
                highest_category = None;
                continue;
            }
            let category = AttrCategory::from_prop(prop);

            if let Some(highest_category_value) = highest_category
                && category < highest_category_value
            {
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

            highest_category =
                Some(highest_category.map_or(category, |highest| highest.max(category)));
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
    fn directives_participate_in_ordering() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div @click="onClick" v-if="show"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_late_directive_groups() {
        let linter = create_linter();
        for source in [
            r#"<div class="panel" v-if="show"></div>"#,
            r#"<button @click="save" :disabled="busy"></button>"#,
            r#"<CopyButton showLabel v-model:copied="copied" />"#,
            r#"<div class="panel" v-ripple></div>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 1, "{source}");
        }
    }

    #[test]
    fn tracks_highest_seen_category() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<Comp @keydown.meta.s.prevent="save" :show="show" :size="size"></Comp>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn object_v_bind_resets_ordering() {
        let linter = create_linter();
        for source in [
            r#"<Comp v-bind="attrs" v-model="value" />"#,
            r#"<Comp v-bind="attrs" :id="id" />"#,
            r#"<Comp v-bind="attrs" :is="kind" />"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(result.warning_count, 0, "{source}");
        }
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
    fn static_attributes_still_enforce_category_order() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="list" id="list"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn static_vue_two_slot_attributes_use_unique_group() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div slot="header" class="panel"></div><div class="panel" slot-scope="scope"></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn glyph_output_lints_clean_for_ordered_directives_and_static_slots() {
        let cases = [
            r#"<div v-show="open" v-once id="help" v-tooltip:dialog="tip" class="_button"></div>"#,
            r#"<Comp #default="{ x }" :data="d"></Comp>"#,
            r#"<div class="panel" slot="header"></div>"#,
        ];

        for source in cases {
            let formatted =
                vize_glyph::format_template(source, &vize_glyph::FormatOptions::default())
                    .expect("formatting must succeed");
            let result = create_linter().lint_template(&formatted, "test.vue");
            assert_eq!(result.warning_count, 0, "{formatted}");
        }
    }
}
