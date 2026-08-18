//! vue/v-slot-style
//!
//! Enforce `v-slot` directive style.
//!
//! ## Options
//!
//! The style is chosen per *position*, not once for the whole rule, because the
//! default slot on a component reads best without any argument at all:
//!
//! - `at_component` (default `v-slot`): the default slot written directly on a
//!   component, as in `<MyComponent v-slot="props">`.
//! - `default_slot` (default `shorthand`): the default slot on a `<template>`,
//!   as in `<template #default="props">`.
//! - `named` (default `shorthand`): any named slot, as in `<template #header>`.
//!   A dynamic argument (`#[name]`) is a named slot.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template v-slot:header>...</template>
//! <MyComponent #default="props">...</MyComponent>
//! ```
//!
//! ### Valid
//! ```vue
//! <template #header>...</template>
//! <MyComponent v-slot="props">...</MyComponent>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

mod position;

use position::{SlotPosition, actual_style, argument_text, slot_argument, slot_position};

static META: RuleMeta = RuleMeta {
    name: "vue/v-slot-style",
    description: "Enforce `v-slot` directive style",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Style preference for one `v-slot` position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VSlotStyleOption {
    /// `#name`
    #[default]
    Shorthand,
    /// `v-slot:name`
    Longform,
    /// `v-slot`, with no argument at all
    VSlot,
}

/// Enforce v-slot directive style
pub struct VSlotStyle {
    /// Style for the default slot written on a component.
    pub at_component: VSlotStyleOption,
    /// Style for the default slot written on a `<template>`.
    pub default_slot: VSlotStyleOption,
    /// Style for a named slot.
    pub named: VSlotStyleOption,
}

impl Default for VSlotStyle {
    fn default() -> Self {
        Self {
            at_component: VSlotStyleOption::VSlot,
            default_slot: VSlotStyleOption::Shorthand,
            named: VSlotStyleOption::Shorthand,
        }
    }
}

impl VSlotStyle {
    /// One style for every position, the shape of the single-string option.
    pub fn uniform(style: VSlotStyleOption) -> Self {
        Self {
            at_component: style,
            default_slot: style,
            named: style,
        }
    }

    fn expected(&self, position: SlotPosition) -> VSlotStyleOption {
        match position {
            SlotPosition::AtComponent => self.at_component,
            SlotPosition::DefaultOnTemplate => self.default_slot,
            SlotPosition::Named => self.named,
        }
    }
}

impl Rule for VSlotStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "slot" {
            return;
        }
        let argument = directive.arg.as_ref().and_then(|arg| match arg {
            ExpressionNode::Simple(simple) if simple.is_static => Some(simple.content),
            _ => None,
        });
        let expected = self.expected(slot_position(element, &directive.arg, argument));
        let actual = actual_style(ctx.source, directive);
        if actual == expected {
            return;
        }

        let written = slot_argument(ctx.source, directive).unwrap_or("v-slot");
        let name = argument_text(ctx.source, directive).unwrap_or("default");
        let message = match expected {
            VSlotStyleOption::Shorthand => ctx.t_fmt(
                "vue/v-slot-style.message_shorthand",
                &[("name", name), ("actual", written)],
            ),
            VSlotStyleOption::Longform => ctx.t_fmt(
                "vue/v-slot-style.message_longform",
                &[("name", name), ("actual", written)],
            ),
            VSlotStyleOption::VSlot => {
                ctx.t_fmt("vue/v-slot-style.message_v_slot", &[("actual", written)])
            }
        };
        let help = ctx.t("vue/v-slot-style.help");
        ctx.warn_with_help(message, &directive.loc, help);
    }
}

#[cfg(test)]
mod tests;
