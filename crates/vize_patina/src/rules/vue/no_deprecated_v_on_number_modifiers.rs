//! vue/no-deprecated-v-on-number-modifiers
//!
//! Disallow numeric `keyCode` modifiers on `v-on` (deprecated and removed in
//! Vue 3).
//!
//! In Vue 2, you could bind a keyboard handler to a specific physical key by its
//! `keyCode` directly in the modifier, e.g. `@keyup.13="submit"` (13 is Enter)
//! or `@keyup.27="cancel"` (27 is Escape). Vue 3 removed numeric `keyCode`
//! modifiers entirely in favour of named keys derived from `KeyboardEvent.key`,
//! e.g. `@keyup.enter` and `@keyup.esc`. A lingering numeric modifier in a Vue 3
//! template is no longer interpreted as a key filter, so the handler fires for
//! every key (or not at all) instead of only the intended one.
//!
//! This mirrors eslint-plugin-vue's `vue/no-deprecated-v-on-number-modifiers`.
//!
//! ## Dialect gating
//!
//! The rule fires only for the default Vue 3 dialect. petite-vue and the legacy
//! Vue 2 / 2.7 dialect still understand numeric `keyCode` modifiers, so neither
//! should be flagged here.
//!
//! ## Examples
//!
//! ### Invalid (Vue 3)
//! ```vue
//! <input @keyup.13="submit" />
//! <input v-on:keyup.27="cancel" />
//! <input @keyup.13.stop="submit" />
//! ```
//!
//! ### Valid (Vue 3)
//! ```vue
//! <input @keyup.enter="submit" />
//! <input @keyup.esc="cancel" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::dialect::VueDialect;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-deprecated-v-on-number-modifiers",
    description: "Disallow deprecated numeric `keyCode` modifiers on `v-on`",
    category: RuleCategory::Essential,
    fixable: true,
    default_severity: Severity::Error,
};

/// Disallow deprecated numeric `keyCode` modifiers on `v-on`.
pub struct NoDeprecatedVOnNumberModifiers;

impl Rule for NoDeprecatedVOnNumberModifiers {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only the default Vue 3 dialect removed numeric `keyCode` modifiers.
        // petite-vue and the legacy Vue 2 dialect still understand them, so
        // leave them alone.
        if ctx.dialect() != VueDialect::Vue {
            return;
        }

        // Only `v-on` / `@` carries key-filter modifiers.
        if directive.name != "on" {
            return;
        }

        // eslint-plugin-vue selects the first parseInt-compatible modifier before
        // exempting 0–9. In particular, `.0.13` is clean, while `.13.27`
        // reports only `13`; scanning for every deprecated value changes that
        // contract. Numeric prefixes such as `13suffix` are included too.
        let Some((modifier, key_code)) = directive.modifiers.iter().find_map(|modifier| {
            decimal_key_code(modifier.content).map(|key_code| (modifier, key_code))
        }) else {
            return;
        };
        if (0.0..=9.0).contains(&key_code) {
            return;
        }

        let mut diagnostic = LintDiagnostic::error(
            META.name,
            ctx.t_fmt(
                "vue/no-deprecated-v-on-number-modifiers.message",
                &[("keyCode", modifier.content)],
            ),
            modifier.loc.span.start,
            modifier.loc.span.end,
        );
        if let Some(help) = ctx
            .help_level()
            .process(&ctx.t("vue/no-deprecated-v-on-number-modifiers.help"))
        {
            diagnostic = diagnostic.with_help(help);
        }
        if let Some(key) = named_key(key_code) {
            diagnostic = diagnostic.with_fix(Fix::new(
                "Replace the numeric keyCode modifier with a named key",
                TextEdit::replace(modifier.loc.span.start, modifier.loc.span.end, key),
            ));
        }
        ctx.report(diagnostic);
    }
}

/// Match Number.isInteger(Number.parseInt(name, 10)), including prefix parsing,
/// signed zero, IEEE-754 rounding and rejection of infinite overflow.
fn decimal_key_code(name: &str) -> Option<f64> {
    // ECMAScript WhiteSpace + LineTerminator (Rust's is_whitespace additionally
    // accepts U+0085 and omits U+FEFF).
    let name = name.trim_start_matches(|c: char| {
        matches!(
            c,
            '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
                ..='\u{200a}'
                    | '\u{2028}'
                    | '\u{2029}'
                    | '\u{202f}'
                    | '\u{205f}'
                    | '\u{3000}'
                    | '\u{feff}'
        )
    });
    let sign_len = usize::from(name.starts_with(['+', '-']));
    let digits = name
        .as_bytes()
        .get(sign_len..)?
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }
    let value = name.get(..sign_len + digits)?.parse::<f64>().ok()?;
    value.is_finite().then_some(value)
}

/// eslint-plugin-vue 10.9.2's keycode-to-key mapping. Codes 8 and 9 are
/// deliberately unreachable because its first numeric modifier gate exempts 0–9.
fn named_key(key_code: f64) -> Option<&'static str> {
    const LETTERS: [&str; 26] = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];
    const FUNCTION_KEYS: [&str; 12] = [
        "f1", "f2", "f3", "f4", "f5", "f6", "f7", "f8", "f9", "f10", "f11", "f12",
    ];
    if (65.0..=90.0).contains(&key_code) {
        return LETTERS.get(key_code as usize - 65).copied();
    }
    if (112.0..=123.0).contains(&key_code) {
        return FUNCTION_KEYS.get(key_code as usize - 112).copied();
    }
    Some(match key_code {
        13.0 => "enter",
        16.0 => "shift",
        17.0 => "ctrl",
        18.0 => "alt",
        19.0 => "pause",
        20.0 => "caps-lock",
        27.0 => "escape",
        32.0 => "space",
        33.0 => "page-up",
        34.0 => "page-down",
        35.0 => "end",
        36.0 => "home",
        37.0 => "arrow-left",
        38.0 => "arrow-up",
        39.0 => "arrow-right",
        40.0 => "arrow-down",
        45.0 => "insert",
        46.0 => "delete",
        144.0 => "num-lock",
        145.0 => "scroll-lock",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{NoDeprecatedVOnNumberModifiers, decimal_key_code};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDeprecatedVOnNumberModifiers));
        Linter::with_registry(registry)
    }

    /// Wrap markup in a petite-vue document so `ctx.dialect()` resolves to
    /// petite-vue and the rule gates itself off.
    fn petite_doc(markup: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
  <body>
    <div v-scope="{{ count: 0 }}">
{markup}
    </div>
    <script src="https://unpkg.com/petite-vue" init></script>
  </body>
</html>"#
        )
    }

    #[test]
    fn reports_number_modifier_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.13="submit" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_number_modifier_full_syntax() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-on:keyup.27="cancel" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_number_combined_with_other_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.13.stop="submit" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_first_number_modifier() {
        // The provider selects only the first parseInt-compatible modifier.
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.13.27="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn allows_named_key_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<input @keyup.enter="submit" @keyup.esc="cancel" />"#,
            "App.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn ignores_alphanumeric_modifier() {
        // A modifier that merely contains digits but is not purely numeric must
        // not be flagged (e.g. an `f2`-style named key).
        let linter = create_linter();
        let result = linter.lint_template(r#"<input @keyup.f2="handler" />"#, "App.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn qualifies_complete_provider_matrix_and_malformed_control() {
        // Independently captured, unfiltered ESLint results, including fixes.
        // These are isolated-rule findings, not n8n full-configuration acceptance.
        let packet: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../docs/acceptance/numeric-key-modifiers/provider.json"
        ))
        .unwrap();
        let linter = create_linter();
        for case in packet["records"].as_array().unwrap() {
            let source = case["source"].as_str().unwrap();
            let provider = &case["results"][0];
            let expected = provider["messages"].as_array().unwrap();
            let result = linter.lint_sfc(source, "Numeric.vue");
            if case["modifier"] == "" {
                // Keep the complete malformed control: Vue rejects a trailing
                // dot before the numeric rule runs, while the provider is clean.
                // This is a documented parser boundary, not qualified parity.
                assert!(expected.is_empty());
                assert_eq!(result.error_count, 1);
                assert_eq!(result.diagnostics.len(), 1);
                assert_eq!(result.diagnostics[0].rule_name, "parser/template");
                assert!(result.diagnostics[0].fix.is_none());
                continue;
            }
            assert_eq!(result.error_count, expected.len(), "{source}");
            assert_eq!(result.diagnostics.len(), expected.len(), "{source}");
            for (actual, expected) in result.diagnostics.iter().zip(expected) {
                assert_eq!(actual.rule_name, expected["ruleId"].as_str().unwrap());
                assert_eq!(actual.severity, crate::diagnostic::Severity::Error);
                assert_eq!(expected["severity"], 2);
                // Probe inputs are ASCII and on one line; ESLint's 1-based
                // columns and UTF-16 ranges therefore map directly to bytes.
                assert_eq!(
                    actual.start as u64 + 1,
                    expected["column"].as_u64().unwrap(),
                    "{source}"
                );
                assert_eq!(
                    actual.end as u64 + 1,
                    expected["endColumn"].as_u64().unwrap(),
                    "{source}"
                );
                if let Some(fix) = actual.fix.as_ref() {
                    let expected_fix = &expected["fix"];
                    assert_eq!(fix.edits.len(), 1, "{source}");
                    let edit = &fix.edits[0];
                    assert_eq!(
                        edit.start as u64,
                        expected_fix["range"][0].as_u64().unwrap(),
                        "{source}"
                    );
                    assert_eq!(
                        edit.end as u64,
                        expected_fix["range"][1].as_u64().unwrap(),
                        "{source}"
                    );
                    assert_eq!(
                        edit.new_text.as_str(),
                        expected_fix["text"].as_str().unwrap(),
                        "{source}"
                    );
                } else {
                    assert!(expected.get("fix").is_none(), "{source}");
                }
            }
        }
    }

    #[test]
    fn keeps_physical_sfc_byte_ranges_and_applies_prefix_fix() {
        let source = "<script setup>const title = '日本語';</script>\n<template>\n  <input v-on:keyup.stop.13suffix.27=\"submit\" />\n</template>";
        let result = create_linter().lint_sfc(source, "Numeric.vue");
        assert_eq!(result.error_count, 1);
        let diagnostic = &result.diagnostics[0];
        let start = source.find("13suffix").unwrap() as u32;
        assert_eq!((diagnostic.start, diagnostic.end), (start, start + 8));
        let fix = diagnostic.fix.as_ref().unwrap();
        assert_eq!((fix.edits[0].start, fix.edits[0].end), (start, start + 8));
        assert_eq!(fix.apply(source), source.replace("13suffix", "enter"));
    }

    #[test]
    fn keeps_fix_when_help_is_disabled() {
        let result = create_linter()
            .with_help_level(crate::HelpLevel::None)
            .lint_sfc(
                r#"<template><input @keyup.13="submit" /></template>"#,
                "Numeric.vue",
            );
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].help.is_none());
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn numeric_prefix_parser_uses_javascript_decimal_semantics() {
        for (name, expected) in [
            ("13suffix", Some(13.0)),
            ("+13", Some(13.0)),
            ("-0", Some(-0.0)),
            ("0013", Some(13.0)),
            ("0x13", Some(0.0)),
            ("1e2", Some(1.0)),
            ("\u{feff}13suffix", Some(13.0)),
            ("\u{a0}-13", Some(-13.0)),
            ("\u{85}13", None),
            ("f2", None),
            ("+", None),
            ("", None),
            ("９９", None),
            ("9007199254740993", Some(9007199254740992.0)),
        ] {
            assert_eq!(decimal_key_code(name), expected, "{name}");
        }
        assert_eq!(decimal_key_code(&"9".repeat(309)), None);
    }

    #[test]
    fn ignores_number_modifier_in_petite_vue() {
        // petite-vue still understands numeric keyCode modifiers; never flag it
        // there.
        let linter = create_linter();
        let result =
            linter.lint_standalone_html(&petite_doc(r#"<input @keyup.13="submit">"#), "index.html");
        assert_eq!(result.error_count, 0);
    }
}
