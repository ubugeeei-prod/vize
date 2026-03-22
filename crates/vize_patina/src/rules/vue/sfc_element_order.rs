//! vue/sfc-element-order
//!
//! Enforce a consistent order of top-level elements in SFC.
//!
//! Single-File Components should keep their top-level blocks in a
//! predictable order:
//!
//! 1. `<script>` / `<script setup>`
//! 2. `<template>`
//! 3. `<style>`
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>...</template>
//! <style>...</style>
//! <script setup>...</script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>...</script>
//! <template>...</template>
//! <style></style>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{parse_sfc, BlockLocation, SfcParseOptions};

static META: RuleMeta = RuleMeta {
    name: "vue/sfc-element-order",
    description: "Enforce consistent order of SFC top-level elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

static HELP_ORDER: &str = "Recommended order: <script> -> <template> -> <style>";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SfcElementType {
    Script,
    Template,
    Style,
}

impl SfcElementType {
    #[inline]
    fn order_message(self, previous: Self) -> &'static str {
        match (self, previous) {
            (Self::Script, Self::Template) => "<script> should come before <template>",
            (Self::Script, Self::Style) => "<script> should come before <style>",
            (Self::Template, Self::Style) => "<template> should come before <style>",
            _ => "SFC top-level blocks are out of order",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OrderedBlock {
    kind: SfcElementType,
    start: u32,
    end: u32,
}

impl OrderedBlock {
    #[inline]
    fn new(kind: SfcElementType, loc: &BlockLocation) -> Self {
        Self {
            kind,
            start: loc.tag_start as u32,
            end: loc.tag_end as u32,
        }
    }
}

/// Enforce SFC element order.
pub struct SfcElementOrder;

impl Rule for SfcElementOrder {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let descriptor = match parse_sfc(
            ctx.source,
            SfcParseOptions {
                filename: ctx.filename.into(),
                ..Default::default()
            },
        ) {
            Ok(descriptor) => descriptor,
            Err(_) => return,
        };

        let mut blocks = Vec::with_capacity(2 + descriptor.styles.len());

        if let Some(script) = descriptor.script.as_ref() {
            blocks.push(OrderedBlock::new(SfcElementType::Script, &script.loc));
        }
        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            blocks.push(OrderedBlock::new(SfcElementType::Script, &script_setup.loc));
        }
        if let Some(template) = descriptor.template.as_ref() {
            blocks.push(OrderedBlock::new(SfcElementType::Template, &template.loc));
        }
        for style in &descriptor.styles {
            blocks.push(OrderedBlock::new(SfcElementType::Style, &style.loc));
        }

        blocks.sort_unstable_by_key(|block| block.start);

        for index in 1..blocks.len() {
            let current = blocks[index];
            let previous = blocks[index - 1];

            if current.kind < previous.kind {
                ctx.report(
                    LintDiagnostic::warn(
                        META.name,
                        current.kind.order_message(previous.kind),
                        current.start,
                        current.end,
                    )
                    .with_help(HELP_ORDER),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SfcElementOrder;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(SfcElementOrder));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_order_script_template_style() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<script setup></script>
<template><div></div></template>
<style></style>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_template_before_script() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div></div></template>
<script setup></script>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/sfc-element-order");
        assert!(result.diagnostics[0].message.contains("<script>"));
    }

    #[test]
    fn test_invalid_style_before_template() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<script setup></script>
<style></style>
<template><div></div></template>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/sfc-element-order");
    }

    #[test]
    fn test_custom_blocks_are_ignored_for_ordering() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<docs>hello</docs>
<script setup></script>
<template><div></div></template>
<style></style>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
