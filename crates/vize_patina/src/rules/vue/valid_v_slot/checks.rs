//! Directive validation shared by default and configured slot rule instances.
use super::{
    DirectiveNode, ElementNode, ExpressionNode, LintContext, ValidVSlot, dynamic_binding,
    has_component_parent,
};

impl ValidVSlot {
    pub(super) fn check_slot_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
        allow_modifiers: bool,
    ) {
        if directive.name != "slot" {
            return;
        }

        let tag = element.tag;

        // v-slot can only be used on components or <template>
        if tag != "template" && !Self::is_custom_component(element) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
            return;
        }

        if tag != "template" {
            if Self::is_named_slot(directive) {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-slot.invalid_location"),
                    &directive.loc,
                    ctx.t("vue/valid-v-slot.help"),
                );
            } else if directive.exp.as_ref().is_none_or(
                |exp| matches!(exp, ExpressionNode::Simple(value) if value.content.is_empty()),
            ) {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-slot.missing_value"),
                    &directive.loc,
                    ctx.t("vue/valid-v-slot.value_help"),
                );
            }
        }

        if dynamic_binding::references_own_binding(directive) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.dynamic_scope"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.dynamic_scope_help"),
            );
        }

        // Argument modifiers need an explicit allowance. Argless modifiers are
        // invalid under both settings, matching the official Vue rule.
        if !directive.modifiers.is_empty() && (directive.arg.is_none() || !allow_modifiers) {
            let message = if directive.arg.is_none() {
                "vue/valid-v-slot.invalid_modifier"
            } else {
                "vue/valid-v-slot.named_modifier"
            };
            let help = if directive.arg.is_none() {
                "vue/valid-v-slot.modifier_help"
            } else {
                "vue/valid-v-slot.named_modifier_help"
            };
            ctx.error_with_help(ctx.t(message), &directive.loc, ctx.t(help));
        }

        if tag == "template" && !has_component_parent(ctx) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // Check for duplicate v-slot directives
        let (default_count, named_count) = Self::count_slot_directives(element);

        if default_count > 1 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // On <template>, can only have one named slot
        if tag == "template" && named_count > 1 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // Mixing default and named on same element
        if default_count > 0 && named_count > 0 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }
    }
}
