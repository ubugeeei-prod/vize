//! vue/sfc-element-order
//!
//! Enforce a consistent order of top-level elements in SFC.
//!
//! This is Vize's implementation of `eslint-plugin-vue`'s `vue/block-order`,
//! and it carries that rule's default order — `[["script", "template"], "style"]`.
//! The nested group is what makes `<script>` and `<template>` interchangeable:
//! both orders are idiomatic (the official Vue docs and `create-vue` templates
//! put `<template>` first), so only `<style>` is pinned last. Enforcing a strict
//! `<script>` before `<template>` here would report a warning on the majority of
//! real Vue components that upstream accepts (#3223).
//!
//! 1. `<script>` / `<script setup>` and `<template>`, in either order
//! 2. `<style>`
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
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
//!
//! ```vue
//! <template>...</template>
//! <script setup>...</script>
//! <style></style>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{BlockLocation, SfcParseOptions, parse_sfc};
use vize_s0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/sfc-element-order",
    description: "Enforce consistent order of SFC top-level elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

static HELP_ORDER: &str = "Recommended order: <script> and <template> (either order) -> <style>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SfcElementType {
    Script,
    Template,
    Style,
}

impl SfcElementType {
    /// Rank in `eslint-plugin-vue`'s default `vue/block-order` order,
    /// `[["script", "template"], "style"]`. `Script` and `Template` share a rank
    /// because the nested group makes them interchangeable.
    #[inline]
    fn order_rank(self) -> u8 {
        match self {
            Self::Script | Self::Template => 0,
            Self::Style => 1,
        }
    }

    #[inline]
    fn order_message(self, previous: Self) -> &'static str {
        match (self, previous) {
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
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.sfc_element_order.parse_sfc",
                parse_sfc(
                    ctx.source,
                    SfcParseOptions {
                        filename: ctx.filename.into(),
                        ..Default::default()
                    },
                )
            ) {
                Ok(descriptor) => descriptor,
                Err(_) => return,
            };
            &owned_descriptor
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

        // Upstream anchors each block against the first *earlier* block that
        // outranks it, not merely against its neighbour, so
        // `<style><script><template>` reports both the script and the template.
        // The block count is a handful, so the quadratic scan is free.
        for index in 1..blocks.len() {
            let current = blocks[index];
            let Some(previous) = blocks[..index]
                .iter()
                .copied()
                .find(|block| block.kind.order_rank() > current.kind.order_rank())
            else {
                continue;
            };

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

    /// `eslint-plugin-vue`'s `vue/block-order` default is
    /// `[["script", "template"], "style"]`, so a template-first component — the
    /// shape used by the official Vue docs and by `create-vue`'s templates — is
    /// valid upstream and must stay silent here (#3223).
    #[test]
    fn test_valid_template_before_script() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div></div></template>
<script setup></script>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
        assert!(result.diagnostics.is_empty());
    }

    /// Pinned reproduction from `tests/_fixtures/_git/epic-spinners`
    /// (`packages/docs/src/App.vue`, revision 3a4dda1d). Before #3223 this exact
    /// shape produced one warning per component across 92 corpus projects.
    #[test]
    fn test_pinned_template_first_real_component_stays_clean() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template>
  <loaders-header class="container"/>
  <router-view class="container"/>
  <loaders-footer/>
</template>

<script lang="ts">
export default {
  name: 'app',
}
</script>

<style lang="scss">
.container { margin: 0; }
</style>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_invalid_style_before_script() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<style></style>
<script setup></script>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/sfc-element-order");
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    /// Upstream anchors every block against the first earlier block that
    /// outranks it, so both the script and the template are reported here — not
    /// just the one adjacent to `<style>`.
    #[test]
    fn test_style_first_reports_every_later_block() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<style></style>
<script setup></script>
<template><div></div></template>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "<script> should come before <style>",
                "<template> should come before <style>",
            ],
        );
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
