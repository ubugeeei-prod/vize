//! vue/no-empty-component-block
//!
//! Disallow empty SFC blocks.
//!
//! A `<template>`, `<script>`, or `<style>` block whose content is empty or
//! whitespace-only carries no meaning. Such blocks are usually leftovers from
//! scaffolding or refactoring and should be removed.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template></template>
//!
//! <script></script>
//!
//! <style>
//! </style>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div>Hello</div>
//! </template>
//!
//! <script setup>
//! const message = "Hello";
//! </script>
//!
//! <style scoped>
//! .button { color: red; }
//! </style>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/no-empty-component-block",
    description: "Disallow empty SFC blocks",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// A reported empty block: its kind (for the message variable) and source span.
struct EmptyBlock {
    kind: &'static str,
    start: u32,
    end: u32,
}

/// No empty component block rule.
#[derive(Default)]
pub struct NoEmptyComponentBlock;

impl Rule for NoEmptyComponentBlock {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        // Prefer the descriptor prepared by the engine; only parse the SFC
        // ourselves when one was not shared. `run_on_sfc` never runs for plain
        // template input (that path does not invoke SFC-level rules), so this
        // rule does nothing in that case.
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.no_empty_component_block.parse_sfc",
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

        let mut empty_blocks: Vec<EmptyBlock> = Vec::new();

        if let Some(template) = &descriptor.template
            && template.src.is_none()
            && template.content.trim().is_empty()
        {
            empty_blocks.push(EmptyBlock {
                kind: "template",
                start: template.loc.tag_start as u32,
                end: template.loc.tag_end as u32,
            });
        }

        for script in [&descriptor.script, &descriptor.script_setup]
            .into_iter()
            .flatten()
        {
            if script.src.is_none() && script.content.trim().is_empty() {
                empty_blocks.push(EmptyBlock {
                    kind: "script",
                    start: script.loc.tag_start as u32,
                    end: script.loc.tag_end as u32,
                });
            }
        }

        for style in &descriptor.styles {
            if style.src.is_none() && style.content.trim().is_empty() {
                empty_blocks.push(EmptyBlock {
                    kind: "style",
                    start: style.loc.tag_start as u32,
                    end: style.loc.tag_end as u32,
                });
            }
        }

        if empty_blocks.is_empty() {
            return;
        }

        // Resolve all user-facing strings before reporting so the immutable
        // borrow from `ctx.t*` does not overlap the mutable borrow in `report`.
        let help = ctx.t("vue/no-empty-component-block.help").into_owned();
        let messages: Vec<_> = empty_blocks
            .iter()
            .map(|block| {
                let message = ctx.t_fmt(
                    "vue/no-empty-component-block.message",
                    &[("block", block.kind)],
                );
                (message, block.start, block.end)
            })
            .collect();

        for (message, start, end) in messages {
            ctx.report(
                LintDiagnostic::warn(META.name, message, start, end).with_help(help.clone()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoEmptyComponentBlock;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoEmptyComponentBlock));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_empty_template_block_warns() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template></template>
<script setup>
const a = 1;
</script>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/no-empty-component-block"
        );
    }

    #[test]
    fn test_whitespace_only_template_block_warns() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            "<template>\n   \n</template>\n<script setup>\nconst a = 1;\n</script>\n",
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_empty_script_block_warns() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<script></script>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/no-empty-component-block"
        );
    }

    #[test]
    fn test_empty_style_block_warns() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<style scoped>
</style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_multiple_empty_blocks_each_warn() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template></template>
<script></script>
<style></style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 3);
    }

    #[test]
    fn test_non_empty_blocks_do_not_warn() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template>
  <div>Hello</div>
</template>
<script setup>
const message = "Hello";
</script>
<style scoped>
.button { color: red; }
</style>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_external_src_block_does_not_warn() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r#"<template><div>Hello</div></template>
<script src="./component.js"></script>
"#,
            "Component.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_plain_template_input_does_nothing() {
        let linter = create_linter();
        let result = linter.lint_template("<div>Hello</div>", "Component.vue");
        assert_eq!(result.warning_count, 0);
    }
}
