//! vue/component-name-in-template-casing
//!
//! Enforce specific casing for component names in templates.
//!
//! ## Examples
//!
//! ### Invalid (default: PascalCase)
//! ```vue
//! <my-component />
//! <myComponent />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent />
//! <RouterView />
//! <slot />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::{is_html_tag, is_svg_tag};
use vize_croquis::builtins::is_builtin_component;
use vize_croquis::naming::{is_kebab_case_loose, is_pascal_case};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/component-name-in-template-casing",
    description: "Enforce specific casing for component names in templates",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Casing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentCasing {
    /// PascalCase: MyComponent
    #[default]
    PascalCase,
    /// kebab-case: my-component
    KebabCase,
}

/// Component name in template casing rule
pub struct ComponentNameInTemplateCasing {
    pub casing: ComponentCasing,
}

impl Default for ComponentNameInTemplateCasing {
    fn default() -> Self {
        Self {
            casing: ComponentCasing::PascalCase,
        }
    }
}

impl Rule for ComponentNameInTemplateCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, element, self.casing, false);
    }
}

/// Nuxt-preset variant of [`ComponentNameInTemplateCasing`] that also exempts
/// framework-registered Vuetify 2 components (`v-*` tags) from casing
/// diagnostics.
///
/// Vuetify auto-registers a large set of kebab-case components (`v-btn`,
/// `v-dialog`, ...) that the linter cannot see in source. Enabling this
/// exemption in the Nuxt preset keeps real projects out of a
/// `vue/component-name-in-template-casing` warning storm without loosening
/// the rule for non-Nuxt presets.
pub(crate) struct ComponentNameInTemplateCasingNuxt {
    casing: ComponentCasing,
}

impl Default for ComponentNameInTemplateCasingNuxt {
    fn default() -> Self {
        Self {
            casing: ComponentCasing::PascalCase,
        }
    }
}

impl Rule for ComponentNameInTemplateCasingNuxt {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, element, self.casing, true);
    }
}

fn check_element<'a>(
    ctx: &mut LintContext<'a>,
    element: &ElementNode<'a>,
    casing: ComponentCasing,
    allow_vuetify_tags: bool,
) {
    let tag = element.tag.as_str();

    // Skip HTML elements, SVG elements, and Vue built-ins.
    //
    // Fast path: native tags (div/span/...) contain no uppercase bytes, so
    // their lowercased form is identical. Only allocate via `to_lowercase()`
    // when the tag actually has an uppercase byte, sparing an allocation for
    // every native element (the overwhelmingly common case).
    if tag.bytes().all(|b| !b.is_ascii_uppercase()) {
        if is_html_tag(tag)
            || is_svg_tag(tag)
            || is_builtin_component(tag)
            || is_nuxt_builtin_component(tag)
            || (allow_vuetify_tags && is_vuetify_tag(tag))
        {
            return;
        }
    } else {
        let tag_lower = tag.to_lowercase();
        if is_html_tag(&tag_lower)
            || is_svg_tag(tag)
            || is_builtin_component(tag)
            || is_builtin_component(&tag_lower)
            || is_nuxt_builtin_component(tag)
        {
            return;
        }
    }

    match casing {
        ComponentCasing::PascalCase => {
            if !is_pascal_case(tag) {
                ctx.warn_with_help(
                    ctx.t("vue/component-name-in-template-casing.pascal"),
                    &element.loc,
                    ctx.t("vue/component-name-in-template-casing.help_pascal"),
                );
            }
        }
        ComponentCasing::KebabCase => {
            if !is_kebab_case_loose(tag) {
                ctx.warn_with_help(
                    ctx.t("vue/component-name-in-template-casing.kebab"),
                    &element.loc,
                    ctx.t("vue/component-name-in-template-casing.help_kebab"),
                );
            }
        }
    }
}

fn is_nuxt_builtin_component(tag: &str) -> bool {
    matches!(
        tag,
        "nuxt"
            | "nuxt-child"
            | "nuxt-page"
            | "nuxt-layout"
            | "nuxt-link"
            | "nuxt-loading-indicator"
            | "nuxt-error-boundary"
            | "client-only"
            | "no-ssr"
            | "Nuxt"
            | "NuxtChild"
            | "NuxtPage"
            | "NuxtLayout"
            | "NuxtLink"
            | "NuxtLoadingIndicator"
            | "NuxtErrorBoundary"
            | "ClientOnly"
            | "NoSsr"
    )
}

/// Matches the Vuetify `v-*` tag convention (e.g. `v-btn`, `v-dialog`).
///
/// Vuetify components are framework-registered globally, so they appear in
/// templates without an explicit local import. The linter cannot infer this
/// from source, so the Nuxt preset opts in to treating any tag starting with
/// `v-` followed by a lowercase ASCII letter as a known component name and
/// skips casing/self-closing diagnostics for it.
fn is_vuetify_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    bytes.len() >= 3 && bytes[0] == b'v' && bytes[1] == b'-' && bytes[2].is_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{ComponentNameInTemplateCasing, ComponentNameInTemplateCasingNuxt};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ComponentNameInTemplateCasing::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_pascal_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_kebab_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-component />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_html_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_vue_built_in() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<slot />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_nuxt_child_builtin() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<nuxt-child id="index" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    fn create_nuxt_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ComponentNameInTemplateCasingNuxt::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_nuxt_preset_allows_vuetify_tags() {
        let linter = create_nuxt_linter();
        let result = linter.lint_template(
            r#"<v-dialog><v-btn /><v-icon /><v-spacer /></v-dialog>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_default_still_flags_vuetify_tags() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<v-btn />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_nuxt_preset_still_flags_other_kebab() {
        let linter = create_nuxt_linter();
        let result = linter.lint_template(r#"<my-component />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
