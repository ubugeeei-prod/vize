//! vue/valid-template-root
//!
//! Enforce that an SFC `<template>` block has valid root content.
//!
//! Vue 3 templates compile to fragments, so multiple roots, root `v-for`, root
//! `<slot>`, and bare `<template>` wrappers are valid syntax here. Redundant
//! wrappers belong to `vue/no-lone-template`. This rule instead mirrors the
//! current eslint-plugin-vue contract at the SFC boundary: an inline template
//! needs rendered content, while a template with `src` must not also contain
//! inline content.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div>content</div>
//! </template>
//! ```
//! ```vue
//! <template>
//!   <slot />
//! </template>
//! ```
//! ```vue
//! <template>
//!   <template v-if="ok"><div>a</div></template>
//! </template>
//! ```
//! ```vue
//! <template>
//!   <header>a</header>
//!   <main>b</main>
//! </template>
//! ```
//! ```vue
//! <template>
//!   <div v-if="ok">a</div>
//!   <div v-else>b</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::profile;

static META: RuleMeta = RuleMeta {
    name: "vue/valid-template-root",
    description: "Enforce valid root content in an SFC `<template>` block",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce a valid `<template>` root.
pub struct ValidTemplateRoot;

impl Rule for ValidTemplateRoot {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let owned_descriptor;
        let descriptor = if let Some(descriptor) = ctx.sfc_descriptor() {
            descriptor
        } else {
            owned_descriptor = match profile!(
                "patina.rule.valid_template_root.parse_sfc",
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
        let Some(template) = descriptor.template.as_ref() else {
            return;
        };

        let has_src = template.src.is_some() || template.attrs.contains_key(":src");
        let has_inline_content = !template.content.trim().is_empty();
        let (message, start, end) = if has_src && has_inline_content {
            (
                ctx.t("vue/valid-template-root.non_empty_src"),
                template.loc.start as u32,
                template.loc.end as u32,
            )
        } else if !has_src && !has_inline_content {
            (
                ctx.t("vue/valid-template-root.no_child"),
                template.loc.tag_start as u32,
                template.loc.tag_end as u32,
            )
        } else {
            return;
        };

        ctx.report(
            LintDiagnostic::error(META.name, message, start, end)
                .with_help(ctx.t("vue/valid-template-root.help")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ValidTemplateRoot;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidTemplateRoot));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_single_root() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>content</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_multiple_roots() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<header>a</header><main>b</main>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_v_if_v_else_roots() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div v-if="ok">a</div><div v-else>b</div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_text_root() {
        let linter = create_linter();
        let result = linter.lint_template(r#"hello {{ name }}"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_nested_template() {
        // A `<template>` that is *not* a root (it has a wrapping element) is the
        // job of `vue/no-lone-template`, not this rule.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div><template v-if="ok"><span>a</span></template></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_nested_slot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><slot /></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_slot_root() {
        // `<template><slot/></template>` is the canonical Nuxt layout and renders
        // the slot content under Vue 3 fragment semantics.
        let linter = create_linter();
        let result = linter.lint_template(r#"<slot />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_slot_root_among_siblings() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>a</div><slot />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_template_fragment_root() {
        // A `<template>` root carrying a structural directive is a valid Vue 3
        // fragment (e.g. the common `<template v-if>` / `<template v-else>` pair).
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-if="ok"><div>a</div></template><template v-else><div>b</div></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_template_for_root() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<template v-for="x in xs" :key="x"><li>{{ x }}</li></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_bare_template_root_is_owned_by_no_lone_template() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<template><div>content</div></template>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_empty_sfc_template() {
        let result = create_linter().lint_sfc("<template></template>", "test.vue");
        assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].rule_name, "vue/valid-template-root");
    }

    #[test]
    fn test_valid_external_sfc_template_without_inline_content() {
        let result =
            create_linter().lint_sfc(r#"<template src="./template.html"></template>"#, "test.vue");
        assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_external_sfc_template_with_inline_content() {
        let result = create_linter().lint_sfc(
            r#"<template src="./template.html"><div /></template>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    }
}
