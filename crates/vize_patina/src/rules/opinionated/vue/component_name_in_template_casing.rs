//! vue/component-name-in-template-casing
//!
//! Enforce specific casing for component names in templates.
//!
//! ## Examples
//!
//! ### Invalid (default: PascalCase)
//! ```vue
//! <script setup>import MyComponent from "./MyComponent.vue";</script>
//! <template><my-component /><myComponent /></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent />
//! <RouterView />
//! <slot />
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::naming::{is_kebab_case_loose, is_pascal_case, to_pascal_case};
use vize_l0::{is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_relief::ElementNode;

mod fix;

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
    pub registered_components_only: bool,
    pub globals: Vec<vize_l0::String>,
}

impl ComponentNameInTemplateCasing {
    pub const fn new(casing: ComponentCasing) -> Self {
        Self {
            casing,
            registered_components_only: true,
            globals: Vec::new(),
        }
    }

    /// Check every custom tag, including standalone template blocks.
    pub fn with_registered_components_only(mut self, value: bool) -> Self {
        self.registered_components_only = value;
        self
    }

    /// Additional explicitly registered global component names.
    pub fn with_globals(mut self, globals: Vec<vize_l0::String>) -> Self {
        self.globals = globals
            .into_iter()
            .map(|name| to_pascal_case(&name))
            .collect();
        self
    }
}

impl Default for ComponentNameInTemplateCasing {
    fn default() -> Self {
        Self::new(ComponentCasing::PascalCase)
    }
}

impl Rule for ComponentNameInTemplateCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, element, self, false);
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
    policy: ComponentNameInTemplateCasing,
}

impl ComponentNameInTemplateCasingNuxt {
    pub(crate) const fn new(casing: ComponentCasing) -> Self {
        Self {
            policy: ComponentNameInTemplateCasing::new(casing),
        }
    }

    pub(crate) fn with_policy(policy: ComponentNameInTemplateCasing) -> Self {
        Self { policy }
    }
}

impl Default for ComponentNameInTemplateCasingNuxt {
    fn default() -> Self {
        Self::new(ComponentCasing::PascalCase)
    }
}

impl Rule for ComponentNameInTemplateCasingNuxt {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        check_element(ctx, element, &self.policy, true);
    }
}

fn check_element<'a>(
    ctx: &mut LintContext<'a>,
    element: &ElementNode<'a>,
    policy: &ComponentNameInTemplateCasing,
    allow_vuetify_tags: bool,
) {
    let tag = element.tag;

    // Skip HTML elements, SVG elements, and Vue built-ins.
    //
    // Fast path: native tags (div/span/...) contain no uppercase bytes, so
    // their lowercased form is identical. Only allocate via `to_lowercase()`
    // when the tag actually has an uppercase byte, sparing an allocation for
    // every native element (the overwhelmingly common case).
    if tag.bytes().all(|b| !b.is_ascii_uppercase()) {
        if is_html_tag(tag)
            || is_svg_tag(tag)
            || is_math_ml_tag(tag)
            || matches!(tag, "slot" | "component")
            || is_nuxt_builtin_component(tag)
            || (allow_vuetify_tags && is_vuetify_tag(tag))
        {
            return;
        }
    } else {
        let tag_lower = tag.to_lowercase();
        if is_html_tag(&tag_lower)
            || is_svg_tag(tag)
            || is_math_ml_tag(tag)
            || matches!(tag_lower.as_str(), "slot" | "component")
            || is_nuxt_builtin_component(tag)
        {
            return;
        }
    }

    let casing = policy.casing;
    let valid = match casing {
        ComponentCasing::PascalCase => is_pascal_case(tag),
        ComponentCasing::KebabCase => is_kebab_case_loose(tag),
    };
    if valid {
        return;
    }
    if policy.registered_components_only {
        let name = to_pascal_case(tag);
        if !policy
            .globals
            .iter()
            .any(|global| global.as_str() == name.as_str())
            && !ctx.analysis().is_some_and(|analysis| {
                analysis
                    .template_component_registrations
                    .contains(&name, analysis.bindings.is_script_setup)
            })
        {
            return;
        }
    }
    let (message, help) = match casing {
        ComponentCasing::PascalCase => (
            ctx.t("vue/component-name-in-template-casing.pascal"),
            ctx.t("vue/component-name-in-template-casing.help_pascal"),
        ),
        ComponentCasing::KebabCase => (
            ctx.t("vue/component-name-in-template-casing.kebab"),
            ctx.t("vue/component-name-in-template-casing.help_kebab"),
        ),
    };
    let span = element.loc.span;
    let mut diagnostic = LintDiagnostic::warn(ctx.current_rule, message, span.start, span.end);
    if let Some(processed) = ctx.help_level().process(&help) {
        diagnostic = diagnostic.with_help(processed);
    }
    if !ctx.is_petite_vue()
        && !ctx.filename.ends_with(".jsx")
        && !ctx.filename.ends_with(".tsx")
        && let Some(fix) = fix::element_fix(ctx.source, element, casing, &help)
    {
        diagnostic = diagnostic.with_fix(fix);
    }
    ctx.report(diagnostic);
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
    matches!(tag.as_bytes(), [b'v', b'-', third, ..] if third.is_ascii_lowercase())
}

#[cfg(test)]
mod tests;
