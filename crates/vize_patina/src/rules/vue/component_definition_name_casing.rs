//! vue/component-definition-name-casing
//!
//! Enforce a casing for the name a component definition declares.
//!
//! The checked names are the ones eslint-plugin-vue checks: the `name` option
//! of `export default {}` in `<script>`, of an object passed to
//! `defineComponent`, `defineNuxtComponent`, `createApp`, `app.component`,
//! `app.mixin`, `Vue.extend` or `new Vue`, of `defineOptions({})` in
//! `<script setup>`, and the first argument of `app.component(name, ...)`
//! (any object, not only `createApp` results). A name is checked when it is a
//! string literal or a template literal without expressions; computed names
//! are left alone.
//!
//! The option is eslint-plugin-vue's: `"PascalCase"` (the default) or
//! `"kebab-case"`, set as `ruleOptions["vue/component-definition-name-casing"]`.
//! Renaming a component changes how `<KeepAlive include>`, recursive
//! self-references and devtools find it, so the rule does not rewrite the
//! name; the help names the converted one when there is one.
//!
//! The file name is `vue/component-file-name-casing`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script>
//! export default {
//!   name: 'my_component'
//! }
//! </script>
//! ```
//!
//! ```vue
//! <script setup>
//! defineOptions({ name: 'myComponent' })
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script>
//! export default {
//!   name: 'MyComponent'
//! }
//! </script>
//! ```
//!
//! ```vue
//! <script setup>
//! defineOptions({ name: 'MyComponent' })
//! </script>
//! ```

#[cfg(test)]
mod tests;

use super::component_definitions::{
    DefinitionSink,
    casing::{is_kebab, is_pascal, kebab_case, pascal_case},
    scan_block,
};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::String;

static META: RuleMeta = RuleMeta {
    name: "vue/component-definition-name-casing",
    description: "Enforce PascalCase or kebab-case for component definition names",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Casing a component definition name must use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ComponentDefinitionNameCasingOption {
    /// `MyComponent`.
    #[default]
    PascalCase,
    /// `my-component`.
    KebabCase,
}

impl ComponentDefinitionNameCasingOption {
    const fn label(self) -> &'static str {
        match self {
            Self::PascalCase => "PascalCase",
            Self::KebabCase => "kebab-case",
        }
    }

    fn accepts(self, name: &str) -> bool {
        match self {
            Self::PascalCase => is_pascal(name),
            Self::KebabCase => is_kebab(name),
        }
    }

    /// Upstream's exact converter: the converted name, when it passes.
    fn convert(self, name: &str) -> Option<String> {
        let converted = match self {
            Self::PascalCase => pascal_case(name),
            Self::KebabCase => kebab_case(name),
        };
        self.accepts(&converted).then_some(converted)
    }
}

/// Enforce a casing for component definition names.
#[derive(Debug, Default)]
pub struct ComponentDefinitionNameCasing {
    /// The accepted casing; PascalCase unless configured.
    pub casing: ComponentDefinitionNameCasingOption,
}

struct Finding {
    name: String,
    start: u32,
    end: u32,
}

struct Findings {
    casing: ComponentDefinitionNameCasingOption,
    found: Vec<Finding>,
}

impl DefinitionSink for Findings {
    fn definition(&mut self) {}

    fn name(&mut self, name: &str, start: u32, end: u32) {
        if !self.casing.accepts(name) {
            self.found.push(Finding {
                name: name.into(),
                start,
                end,
            });
        }
    }
}

impl Rule for ComponentDefinitionNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::NONE
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Findings {
            casing: self.casing,
            found: Vec::new(),
        };
        {
            let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts())
            else {
                return;
            };
            if let Some(script) = descriptor.script.as_ref()
                && may_name_component(script.content.as_bytes())
                && let Some(parsed) = scripts.script()
            {
                scan_block(script, parsed, false, &mut findings);
            }
            if let Some(script) = descriptor.script_setup.as_ref()
                && may_name_component(script.content.as_bytes())
                && let Some(parsed) = scripts.script_setup()
            {
                scan_block(script, parsed, true, &mut findings);
            }
        }
        let casing = self.casing.label();
        for finding in findings.found {
            let message = ctx.t_fmt(
                "vue/component-definition-name-casing.message",
                &[("name", finding.name.as_str()), ("casing", casing)],
            );
            let help = match self.casing.convert(&finding.name) {
                Some(suggestion) => ctx.t_fmt(
                    "vue/component-definition-name-casing.help",
                    &[("suggestion", suggestion.as_str())],
                ),
                None => ctx.t_fmt(
                    "vue/component-definition-name-casing.help_unconvertible",
                    &[("casing", casing)],
                ),
            };
            ctx.warn_at_with_help(
                message,
                ByteRange {
                    start: finding.start,
                    end: finding.end,
                },
                help,
            );
        }
    }
}

/// Whether a script can hold a checked name, before it is parsed or walked:
/// a `name` key followed by `:` (quoted or computed keys included), or a
/// `component` member after a `.`. Comments may sit where whitespace may.
fn may_name_component(source: &[u8]) -> bool {
    // Both identifiers and string keys can encode any checked character.
    // Leave decoding (including string line continuations) to the parser.
    if memchr::memchr(b'\\', source).is_some() {
        return true;
    }
    let mut has_line_comment = None;
    memchr::memmem::find_iter(source, b"name").any(|at| is_name_key_end(source, at + "name".len()))
        || memchr::memmem::find_iter(source, b"component").any(|at| {
            is_member_name(source, at)
                // A reverse byte scan cannot distinguish a line comment's
                // text from code. Admit it without rescanning for every name.
                || *has_line_comment
                    .get_or_insert_with(|| memchr::memmem::find(source, b"//").is_some())
        })
}

fn is_name_key_end(source: &[u8], mut at: usize) -> bool {
    if matches!(source.get(at), Some(b'\'' | b'"' | b'`')) {
        at += 1;
    }
    loop {
        match source.get(at) {
            Some(byte) if byte.is_ascii_whitespace() => at += 1,
            // A computed string key can have trivia before its closing bracket.
            Some(b']') => at += 1,
            // A non-ASCII boundary may be JavaScript whitespace. The AST
            // remains authoritative, so conservatively admit it.
            Some(byte) if !byte.is_ascii() => return true,
            Some(b':') => return true,
            // A comment before the colon; let the parse decide.
            Some(b'/') => return matches!(source.get(at + 1), Some(b'*' | b'/')),
            _ => return false,
        }
    }
}

fn is_member_name(source: &[u8], at: usize) -> bool {
    let before = source.get(..at).unwrap_or_default();
    let mut rest = before
        .iter()
        .rev()
        .skip_while(|byte| byte.is_ascii_whitespace());
    match rest.next() {
        // `.component` and `?.component`.
        Some(b'.') => true,
        // A block comment before it; let the parse decide.
        Some(b'/') => rest.next() == Some(&b'*'),
        Some(byte) if !byte.is_ascii() => true,
        _ => false,
    }
}
