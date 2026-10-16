//! vue/valid-v-on
//!
//! Enforce valid `v-on` directives.
//!
//! `v-on` must:
//! - Have an event name (argument)
//! - Have a handler expression (unless using object syntax)
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-on></div>
//! <div @></div>
//! <div @click></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div @click="handleClick"></div>
//! <div v-on:click="handleClick"></div>
//! <div v-on="{ click: handleClick }"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-on",
    description: "Enforce valid `v-on` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-on directives
pub struct ValidVOn;

const JAVASCRIPT_KEYWORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "throw",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

impl Rule for ValidVOn {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        self.check_on_directive(ctx, directive, &[]);
    }
}

impl ValidVOn {
    /// Custom modifier names supplement the built-in and keyboard modifiers.
    pub fn with_modifiers(modifiers: Vec<vize_l0::String>) -> impl Rule {
        ConfiguredValidVOn { modifiers }
    }

    fn check_on_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        directive: &DirectiveNode<'a>,
        custom_modifiers: &[vize_l0::String],
    ) {
        if directive.name != "on" {
            return;
        }

        for modifier in &directive.modifiers {
            if !is_valid_modifier(modifier.content, custom_modifiers) {
                ctx.error_with_help(
                    ctx.t_fmt(
                        "vue/valid-v-on.unsupported_modifier",
                        &[("modifier", modifier.content)],
                    ),
                    &modifier.loc,
                    ctx.t("vue/valid-v-on.modifier_help"),
                );
            }
        }

        let has_arg = directive.arg.is_some();
        let has_exp = directive
            .exp
            .as_ref()
            .map(|e| !is_empty_expression(e))
            .unwrap_or(false);

        if directive.exp.as_ref().is_some_and(is_keyword_expression) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-on.invalid_handler"),
                &directive.loc,
                ctx.t("vue/valid-v-on.help"),
            );
            return;
        }

        // Object syntax: v-on="{ click: handler }"
        if !has_arg && has_exp {
            // This is valid object syntax
            return;
        }

        // Event syntax: @click="handler"
        if has_arg {
            if !has_exp {
                // Only action modifiers provide behavior without a handler.
                let has_verb = directive
                    .modifiers
                    .iter()
                    .any(|modifier| matches!(modifier.content, "stop" | "prevent"));
                if !has_verb {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-on.missing_event"),
                        &directive.loc,
                        ctx.t("vue/valid-v-on.help"),
                    );
                }
            }
            return;
        }

        // No argument and no expression
        ctx.error_with_help(
            ctx.t("vue/valid-v-on.missing_event"),
            &directive.loc,
            ctx.t("vue/valid-v-on.help"),
        );
    }
}

struct ConfiguredValidVOn {
    modifiers: Vec<vize_l0::String>,
}

impl Rule for ConfiguredValidVOn {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        ValidVOn.check_on_directive(ctx, directive, &self.modifiers);
    }
}

#[path = "valid_v_on/key_aliases.rs"]
mod key_aliases;

const VALID_MODIFIERS: &[&str] = &[
    "stop", "prevent", "capture", "self", "ctrl", "shift", "alt", "meta", "native", "once", "left",
    "right", "middle", "passive", "esc", "tab", "enter", "space", "up", "down", "delete", "exact",
];

fn is_valid_modifier(modifier: &str, custom: &[vize_l0::String]) -> bool {
    VALID_MODIFIERS.contains(&modifier)
        || modifier.chars().count() == 1
        || key_aliases::KEY_ALIASES.binary_search(&modifier).is_ok()
        || custom.iter().any(|name| name == modifier)
        || has_integer_prefix(modifier)
}

/// Match Number.isInteger(Number.parseInt(value, 10)), including trailing text,
/// signs, ECMAScript whitespace, and overflow to Infinity. Numeric deprecation
/// diagnostics belong to the separate no-deprecated-v-on-number-modifiers rule.
fn has_integer_prefix(value: &str) -> bool {
    let value = value.trim_start_matches(|c| {
        matches!(c,
            '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' |
            '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
            '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    });
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    let len = digits.bytes().take_while(u8::is_ascii_digit).count();
    len > 0 && digits[..len].parse::<f64>().is_ok_and(f64::is_finite)
}

/// Check if expression is empty
fn is_empty_expression(exp: &ExpressionNode) -> bool {
    match exp {
        ExpressionNode::Simple(s) => s.content.trim().is_empty(),
        ExpressionNode::Compound(c) => c.children.is_empty(),
    }
}

fn is_keyword_expression(exp: &ExpressionNode) -> bool {
    let ExpressionNode::Simple(simple) = exp else {
        return false;
    };

    let candidate = simple.content.trim();
    JAVASCRIPT_KEYWORDS.contains(&candidate)
}

#[cfg(test)]
#[path = "valid_v_on/tests.rs"]
mod tests;
