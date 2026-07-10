//! vue/no-reserved-component-names
//!
//! Disallow the use of reserved names as component names.
//!
//! HTML element names, SVG element names, and Vue built-in component names
//! should not be used as component names.
//!
//! This rule checks explicit component-name declarations (`name` option or
//! `defineOptions({ name })`), NOT names inferred from filenames and NOT names
//! of other components used in the template. This matches the behavior of
//! eslint-plugin-vue. Using `<Transition>` or `<KeepAlive>` in a template is
//! perfectly valid — they are Vue built-in components being used correctly.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   name: 'button'
//! }
//! ```
//!
//! ### Valid
//! ```vue
//! <!-- Button.vue -->
//! <script setup></script>
//! <template><div /></template>
//! ```

use self::extract::{define_options_name, find_component_options, name_string_literal};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::script::script_source_type;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::StringLiteral;
use oxc_parser::Parser;
use vize_carton::String;
use vize_carton::is_html_tag;
use vize_croquis::builtins::is_builtin_component;

static META: RuleMeta = RuleMeta {
    name: "vue/no-reserved-component-names",
    description: "Disallow the use of reserved names as component names",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Reserved names that cannot be used (specific edge cases)
const RESERVED_NAMES: &[&str] = &[
    "annotation-xml",
    "color-profile",
    "font-face",
    "font-face-src",
    "font-face-uri",
    "font-face-format",
    "font-face-name",
    "missing-glyph",
];

/// Disallow reserved component names
pub struct NoReservedComponentNames {
    /// Also disallow HTML element names
    pub disallow_html: bool,
    /// Also disallow Vue built-ins
    pub disallow_vue_builtins: bool,
}

impl Default for NoReservedComponentNames {
    fn default() -> Self {
        Self {
            disallow_html: true,
            disallow_vue_builtins: true,
        }
    }
}

impl Rule for NoReservedComponentNames {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let mut findings = Vec::new();

            if let Some(script) = descriptor.script.as_ref() {
                self.collect_script_block_findings(
                    script.content.as_ref(),
                    script.loc.start,
                    &mut findings,
                );
            }
            if let Some(script_setup) = descriptor.script_setup.as_ref() {
                self.collect_script_setup_block_findings(
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    &mut findings,
                );
            }

            findings
        };

        for finding in findings {
            ctx.error_at_with_help(
                ctx.t_fmt(
                    "vue/no-reserved-component-names.message",
                    &[("name", finding.name.as_str())],
                ),
                ByteRange {
                    start: finding.start,
                    end: finding.end,
                },
                ctx.t("vue/no-reserved-component-names.help"),
            );
        }
    }
}

impl NoReservedComponentNames {
    fn collect_script_block_findings(
        &self,
        source: &str,
        offset: usize,
        findings: &mut Vec<ComponentNameFinding>,
    ) {
        // A finding here requires a default-exported component options object
        // carrying a `name` property. Skip the oxc parse entirely when either
        // token is absent so the common case (no Options API `name`) stays a
        // cheap byte scan instead of a full parse per file.
        let bytes = source.as_bytes();
        if memchr::memmem::find(bytes, b"export default").is_none()
            || memchr::memmem::find(bytes, b"name").is_none()
        {
            return;
        }

        let allocator = OxcAllocator::default();
        let parsed = Parser::new(&allocator, source, script_source_type()).parse();
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }

        if let Some(options) = find_component_options(&parsed.program)
            && let Some(name) = name_string_literal(options)
        {
            self.collect_name_finding(name, offset, findings);
        }
    }

    fn collect_script_setup_block_findings(
        &self,
        source: &str,
        offset: usize,
        findings: &mut Vec<ComponentNameFinding>,
    ) {
        // The only `<script setup>` source of an explicit component name is
        // `defineOptions({ name })`. Without that call there is nothing to
        // flag, so avoid parsing files that never reference it.
        if memchr::memmem::find(source.as_bytes(), b"defineOptions").is_none() {
            return;
        }

        let allocator = OxcAllocator::default();
        let parsed = Parser::new(&allocator, source, script_source_type()).parse();
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }

        if let Some(name) = define_options_name(&parsed.program) {
            self.collect_name_finding(name, offset, findings);
        }
    }

    fn collect_name_finding(
        &self,
        name: &StringLiteral<'_>,
        offset: usize,
        findings: &mut Vec<ComponentNameFinding>,
    ) {
        let value = name.value.as_str();
        if !self.is_reserved_component_name(value) {
            return;
        }

        findings.push(ComponentNameFinding {
            name: String::from(value),
            start: offset as u32 + name.span.start,
            end: offset as u32 + name.span.end,
        });
    }

    fn is_reserved_component_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        RESERVED_NAMES.contains(&name_lower.as_str())
            || (self.disallow_html && is_html_tag(name))
            || (self.disallow_vue_builtins
                && (is_builtin_component(&name_lower) || is_builtin_component(name)))
    }
}

struct ComponentNameFinding {
    name: String,
    start: u32,
    end: u32,
}

mod extract;

#[cfg(test)]
mod tests;
