//! vue/require-component-registration
//!
//! Warn when using components that are not explicitly imported or registered.
//!
//! In Vue SFCs, components should be either:
//! - Imported in `<script setup>` (auto-registered)
//! - Registered via `components` option
//! - Global components registered via `app.component()`
//!
//! This rule helps catch typos and missing imports early.
//!
//! ## Configuration
//!
//! This rule can be configured to ignore certain global components:
//! - Built-in components: component, transition, keep-alive, etc.
//! - Explicit project globals, or auto-imports in the selected Nuxt mode
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <template>
//!   <MyButton>Click</MyButton> <!-- Not imported -->
//! </template>
//!
//! <script setup>
//! // MyButton is not imported
//! </script>
//! ```
//!
//! Good:
//! ```vue
//! <template>
//!   <MyButton>Click</MyButton>
//! </template>
//!
//! <script setup>
//! import MyButton from './MyButton.vue'
//! </script>
//! ```

use self::async_setup::async_component_names;
use self::self_name::{define_options_name, file_stem};
use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::builtins::is_builtin_component;
use vize_croquis::is_native_tag;
use vize_croquis::naming::{camelize, capitalize, names_match, to_pascal_case};
use vize_croquis::script_parser::parse_script_setup;
use vize_croquis::{Croquis, Scope, ScopeData, ScopeKind};
use vize_l0::String;
use vize_l0::ToCompactString;
use vize_relief::BindingType;
use vize_relief::{ElementNode, PropNode, RootNode};

use super::component_registration::options_api_registers;

static META: RuleMeta = RuleMeta {
    name: "vue/require-component-registration",
    description: "Require explicit import or registration for components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Components `app.use(router)` registers globally. Callers add them only when
/// `vue-router` resolves from the linted file.
pub const VUE_ROUTER_COMPONENTS: &[&str] = &["RouterLink", "RouterView"];

/// Components Nuxt registers globally even when component directory scanning
/// is disabled. Callers add them only when `nuxt` resolves from the linted
/// file, keeping plain Vue projects strict.
pub const NUXT_BUILTIN_COMPONENTS: &[&str] = &[
    "Nuxt",
    "NuxtChild",
    "NuxtPage",
    "NuxtLayout",
    "NuxtLink",
    "NuxtLoadingIndicator",
    "NuxtErrorBoundary",
    "NuxtWelcome",
    "NuxtIsland",
    "NuxtRouteAnnouncer",
    "ClientOnly",
    "DevOnly",
    "NoSsr",
];

/// Require component registration rule
#[derive(Default)]
pub struct RequireComponentRegistration {
    /// Additional global components to ignore
    pub ignore_globals: Vec<String>,
    /// Whether to check Nuxt auto-imports
    pub nuxt_mode: bool,
    /// Compiled `ignorePatterns`.
    ignore_patterns: Vec<regex::Regex>,
}

impl RequireComponentRegistration {
    /// Create rule with Nuxt mode enabled
    pub fn nuxt() -> Self {
        Self {
            nuxt_mode: true,
            ..Self::default()
        }
    }

    /// Configure project globals and eslint-plugin-vue style `ignorePatterns`.
    pub fn configured(
        ignore_globals: Vec<String>,
        ignore_patterns: &[String],
        nuxt_mode: bool,
    ) -> Self {
        Self {
            ignore_globals,
            nuxt_mode,
            ignore_patterns: patterns::compile(ignore_patterns),
        }
    }

    /// Check if a tag name is a custom component
    fn is_custom_component(&self, tag: &str) -> bool {
        !is_native_tag(tag)
    }

    /// Check if a component is a Vue built-in
    /// Uses croquis builtins for centralized builtin detection
    fn is_builtin(&self, tag: &str) -> bool {
        // Check exact match first (handles PascalCase like "Transition")
        if is_builtin_component(tag) {
            return true;
        }
        // Check lowercase (handles kebab-case like "keep-alive")
        let lower = tag.to_lowercase();
        is_builtin_component(&lower)
    }

    /// Check only globals explicitly selected by project configuration.
    fn is_configured_global(&self, tag: &str) -> bool {
        let kebab = pascal_to_kebab(tag);
        self.ignore_globals.iter().any(|g| {
            g.eq_ignore_ascii_case(tag)
                || g.eq_ignore_ascii_case(&kebab)
                || component_name_matches(tag, g)
        }) || patterns::matches_any(&self.ignore_patterns, tag)
    }

    /// Whether `tag` refers to the component itself.
    ///
    /// A `<script setup>` component can reference itself by its
    /// filename-derived name or by the name declared via
    /// `defineOptions({ name })` — a documented Vue feature for recursive
    /// components that needs no import (importing yourself is not possible
    /// without an alias).
    fn is_self_reference(&self, ctx: &LintContext<'_>, tag: &str) -> bool {
        let stem = file_stem(ctx.filename);
        if !stem.is_empty() && setup_name_matches(tag, capitalize(camelize(stem).as_str()).as_str())
        {
            return true;
        }
        ctx.analysis()
            .and_then(define_options_name)
            .is_some_and(|name| setup_name_matches(tag, name))
    }

    fn is_template_visible_imported_component(
        &self,
        ctx: &LintContext<'_>,
        analysis: &Croquis,
        tag: &str,
    ) -> bool {
        let allow_plain_script_imports = ctx.sfc_descriptor().is_some_and(|descriptor| {
            descriptor.script.is_some() && descriptor.script_setup.is_some()
        });
        analysis
            .scopes
            .iter()
            .filter(|scope| {
                Self::is_value_import_scope(analysis, scope, allow_plain_script_imports)
            })
            .flat_map(|scope| scope.bindings())
            .any(|(name, binding)| {
                matches!(
                    binding.binding_type,
                    BindingType::SetupConst | BindingType::SetupMaybeRef
                ) && template_binding_matches(tag, name)
            })
    }

    fn is_value_import_scope(analysis: &Croquis, scope: &Scope, allow_plain_script: bool) -> bool {
        let ScopeData::ExternalModule(data) = scope.data() else {
            return false;
        };
        if data.is_type_only {
            return false;
        }
        scope
            .parent()
            .and_then(|id| analysis.scopes.get_scope(id))
            .is_some_and(|parent| {
                parent.kind == ScopeKind::ScriptSetup
                    || (allow_plain_script && parent.kind == ScopeKind::NonScriptSetup)
            })
    }

    fn is_options_api_registered_component(&self, analysis: &Croquis, tag: &str) -> bool {
        analysis
            .component_registrations
            .iter()
            .any(|registration| component_name_matches(tag, registration.name.as_str()))
    }
}

impl Rule for RequireComponentRegistration {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Collect all custom components used in template
        let mut used_components: Vec<(String, u32, u32)> = Vec::new();
        collect_components(root, &mut used_components);
        let art_bindings = art::setup_bindings(ctx);
        let async_components = if art_bindings.is_some() {
            Vec::new()
        } else {
            ctx.sfc_descriptor()
                .and_then(|descriptor| descriptor.script_setup.as_ref())
                .map(async_component_names)
                .unwrap_or_default()
        };
        let setup_bindings: Vec<String> = art_bindings.unwrap_or_else(|| {
            ctx.sfc_descriptor()
                .and_then(|descriptor| descriptor.script_setup.as_ref())
                .map(|script| {
                    parse_script_setup(script.content.as_ref())
                        .bindings
                        .iter()
                        .map(|(name, _)| name.to_compact_string())
                        .collect()
                })
                .unwrap_or_default()
        });

        // Registration is project context, never a guess based on a tag prefix.
        for (tag, start, end) in used_components {
            if self.is_custom_component(&tag)
                && !self.is_builtin(&tag)
                && !ctx.is_custom_element(&tag)
                && !self.is_configured_global(&tag)
            {
                // Recursive self-reference needs no registration (#4953).
                if (!art::is_context(ctx) && self.is_self_reference(ctx, &tag))
                    || art::is_target(ctx, &tag)
                {
                    continue;
                }

                if art::analysis(ctx).is_some_and(|analysis| {
                    self.is_template_visible_imported_component(ctx, analysis, &tag)
                        || self.is_options_api_registered_component(analysis, &tag)
                }) || options_api_registers(ctx, &tag)
                    || async_components
                        .iter()
                        .any(|name| setup_name_matches(&tag, name))
                    || setup_bindings
                        .iter()
                        .any(|name| template_binding_matches(&tag, name))
                {
                    continue;
                }

                // In Nuxt mode, don't warn as components are auto-imported
                if self.nuxt_mode {
                    continue;
                }

                ctx.report(
                    LintDiagnostic::warn(
                        META.name,
                        "Component is used but not explicitly imported",
                        start,
                        end,
                    )
                    .with_help("Import the component in <script setup> or register it in components option"),
                );
            }
        }
    }
}

/// Collect all element tags from the template
fn collect_components<'a>(root: &RootNode<'a>, result: &mut Vec<(String, u32, u32)>) {
    fn visit_element<'a>(element: &ElementNode<'a>, result: &mut Vec<(String, u32, u32)>) {
        let start = element.loc.span.start + 1;
        let tag_str = element.tag;
        result.push((
            tag_str.to_compact_string(),
            start,
            start + tag_str.len() as u32,
        ));

        // A static `is` names a component on `<component>`; elsewhere only a
        // `vue:` value does, the rest is a customized built-in element.
        let dynamic_component = matches!(tag_str, "component" | "Component");
        for prop in element.props.iter() {
            let PropNode::Attribute(attribute) = prop else {
                continue;
            };
            if !attribute.name.eq_ignore_ascii_case("is") {
                continue;
            }
            let Some(content) = attribute.value.as_ref().map(|value| value.content) else {
                continue;
            };
            let Some(value) = content
                .strip_prefix("vue:")
                .or(dynamic_component.then_some(content))
            else {
                continue;
            };
            result.push((
                value.to_compact_string(),
                attribute.loc.span.start,
                attribute.loc.span.end,
            ));
        }

        for child in element.children.iter() {
            if let vize_relief::TemplateChildNode::Element(el) = child {
                visit_element(el, result);
            }
        }
    }

    for child in root.children.iter() {
        if let vize_relief::TemplateChildNode::Element(el) = child {
            visit_element(el, result);
        }
    }
}

/// Convert PascalCase to kebab-case
fn pascal_to_kebab(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn component_name_matches(used: &str, registered: &str) -> bool {
    used == registered
        || names_match(used, registered)
        || to_pascal_case(used).as_str() == registered
}

/// Vue's `resolveSetupReference`: the tag as written, camelized, or
/// capitalized after camelizing. Only `-` separates words, so `<my_button>`
/// does not resolve to `MyButton`.
pub(super) fn setup_name_matches(tag: &str, binding: &str) -> bool {
    if tag == binding {
        return true;
    }
    let camel = camelize(tag);
    camel.as_str() == binding || capitalize(camel.as_str()).as_str() == binding
}

fn template_binding_matches(tag: &str, binding: &str) -> bool {
    setup_name_matches(tag, binding)
        || tag
            .split_once('.')
            .is_some_and(|(namespace, _)| setup_name_matches(namespace, binding))
}

mod art;
mod async_setup;
mod patterns;
mod self_name;

#[cfg(test)]
mod tests;
