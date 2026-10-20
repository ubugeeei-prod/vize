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
                    ExpressionNode::Simple(simple) if simple.is_static => Some(simple.content),
                    _ => None,
                });

                match dir.name {
                    "for" => AttrCategory::ListRendering,
                    "if" | "else-if" | "else" | "show" | "cloak" => AttrCategory::Conditionals,
                    "pre" | "once" => AttrCategory::RenderModifiers,
                    "model" => AttrCategory::TwoWayBinding,
                    "on" => AttrCategory::Events,
                    "html" | "text" => AttrCategory::Content,
                    // A bare `#` is an ordinary attribute in the pinned
                    // vue-eslint-parser, unlike `#default` and `v-slot`.
                    // Relief retains the prefix, so named slots need the arg check.
                    "slot" if dir.raw_name == Some("#") && dir.arg.is_none() => {
                        AttrCategory::OtherAttrs
                    }
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

fn is_object_binding(prop: &PropNode<'_>) -> bool {
    matches!(prop, PropNode::Directive(dir) if dir.name == "bind" && dir.arg.is_none())
}

fn is_attribute_or_binding(prop: &PropNode<'_>) -> bool {
    matches!(prop, PropNode::Attribute(_))
        || matches!(prop, PropNode::Directive(dir) if matches!(dir.name, "bind" | "model")
            || (dir.name == "slot" && dir.raw_name == Some("#") && dir.arg.is_none()))
}

fn ignored_object_binding(props: &[PropNode<'_>], index: usize) -> bool {
    props.get(index).is_some_and(is_object_binding)
        && (index + 1 == props.len()
            || index
                .checked_sub(1)
                .and_then(|previous| props.get(previous))
                .is_some_and(is_attribute_or_binding)
            || props.get(index + 1).is_some_and(is_attribute_or_binding))
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
        let mut next_binding = (0, None);

        for (index, prop) in element.props.iter().enumerate() {
            // Vue 3 spread precedence prevents moving an adjacent binding.
            // Skip only that spread; earlier ordering still constrains later props.
            if ignored_object_binding(&element.props, index) {
                continue;
            }
            let category = if is_object_binding(prop) {
                // An isolated spread shares the next non-spread binding's
                // position. Cache the lookahead to scan each prop at most once.
                if next_binding.0 <= index {
                    next_binding = element
                        .props
                        .iter()
                        .enumerate()
                        .skip(index + 1)
                        .find(|(_, next)| is_attribute_or_binding(next) && !is_object_binding(next))
                        .map_or((element.props.len(), None), |(index, next)| {
                            (index, Some(AttrCategory::from_prop(next)))
                        });
                }
                next_binding.1.unwrap_or(AttrCategory::OtherAttrs)
            } else {
                AttrCategory::from_prop(prop)
            };

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
    fn global_id_precedes_unique_key_and_ref() {
        let linter = create_linter();
        let valid = linter.lint_template(
            r#"<div v-for="item in items" :id="item.k" :key="item.k" ref="el" class="x" />"#,
            "test.vue",
        );
        assert_eq!(valid.warning_count, 0, "got: {:?}", valid.diagnostics);

        let invalid = linter.lint_template(
            r#"<div v-for="item in items" :key="item.k" :id="item.k" class="x" />"#,
            "test.vue",
        );
        assert_eq!(invalid.warning_count, 1, "got: {:?}", invalid.diagnostics);
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
    fn adjacent_object_v_bind_does_not_force_its_own_order() {
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
    fn trailing_object_v_bind_preserves_caller_override_precedence() {
        let linter = create_linter();
        for source in [
            r#"<button type="button" class="base-button" @click="onClick" v-bind="attrs" />"#,
            r#"<Comp v-if="show" @finish="finish" v-bind="attrs" />"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, 0,
                "{source}: {:?}",
                result.diagnostics
            );
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
