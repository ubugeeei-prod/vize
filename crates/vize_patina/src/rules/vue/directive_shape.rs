//! The authored shape shared by `v-cloak`, `v-once`, `v-text` and `v-html`:
//! no argument, no modifiers, and a value that is either required or
//! forbidden. Each finding covers the whole directive.

use crate::context::LintContext;
use crate::markup::MarkupDirective;

/// Whether the directive needs a value or must not carry one.
#[derive(Clone, Copy)]
pub(super) enum Value {
    Required,
    Forbidden,
}

/// One rule's directive name, value policy and message keys.
pub(super) struct Shape {
    pub name: &'static str,
    pub value: Value,
    pub value_message: &'static str,
    pub argument_message: &'static str,
    pub modifier_message: &'static str,
    pub help: &'static str,
}

impl Shape {
    pub(super) fn check(&self, ctx: &mut LintContext<'_>, directive: &MarkupDirective<'_>) {
        if !directive.name_eq(self.name) {
            return;
        }
        let range = directive.range();
        let value_wrong = match self.value {
            Value::Required => !directive.has_value(),
            Value::Forbidden => directive.has_value(),
        };
        let mut has_modifier = false;
        directive.walk_modifier_ranges(&mut |_, _| has_modifier = true);
        for (wrong, message) in [
            (value_wrong, self.value_message),
            (directive.argument_range().is_some(), self.argument_message),
            (has_modifier, self.modifier_message),
        ] {
            if wrong {
                ctx.error_at_with_help(ctx.t(message), range, ctx.t(self.help));
            }
        }
    }
}
