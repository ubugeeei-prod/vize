//! vue/no-unused-components
//!
//! Disallow registering components that are not used inside templates.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! import MyButton from './MyButton.vue'  // imported but never used
//! </script>
//!
//! <template>
//!   <div>Hello</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! import MyButton from './MyButton.vue'
//! </script>
//!
//! <template>
//!   <MyButton>Click me</MyButton>
//! </template>
//! ```

mod dynamic_is;
mod script_setup_refs;
#[cfg(test)]
mod tests;

use self::dynamic_is::has_dynamic_is_binding;
use self::script_setup_refs::script_setup_component_import_references;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::script::jsx_referenced_names;
use vize_croquis::ScopeId;
use vize_croquis::naming::{is_pascal_case, to_pascal_case};
use vize_croquis::{Croquis, Scope, ScopeData, ScopeKind};
use vize_l0::cstr;
use vize_l0::{CompactString, FxHashSet, String, ToCompactString};
use vize_relief::BindingType;
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-components",
    description: "Disallow registering components that are not used inside templates",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow unused components
#[derive(Default)]
pub struct NoUnusedComponents {
    /// Pattern for components to ignore (e.g., starts with '_')
    pub ignore_pattern: Option<String>,
}

#[derive(Debug, Clone)]
struct ComponentCandidate {
    name: CompactString,
    local_name: CompactString,
    is_script_setup_import: bool,
    setup_scope: Option<ScopeId>,
    span: Option<(u32, u32)>,
}

impl NoUnusedComponents {
    /// Check if a component name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        // Ignore components starting with underscore
        if name.starts_with('_') {
            return true;
        }

        // Check custom ignore pattern
        if let Some(ref pattern) = self.ignore_pattern
            && name.starts_with(pattern.as_str())
        {
            return true;
        }

        false
    }

    /// Check if an import source should be treated as a Vue component module.
    fn is_component_import_source(source: &str) -> bool {
        let path = source.split(['?', '#']).next().unwrap_or(source);
        path.ends_with(".vue")
    }

    /// Check if an imported binding type indicates a runtime component value.
    fn is_component_binding(binding_type: BindingType) -> bool {
        matches!(binding_type, BindingType::SetupConst)
    }

    fn component_candidates(
        analysis: &Croquis,
        script_source: Option<&str>,
    ) -> Vec<ComponentCandidate> {
        let mut candidates = Vec::new();

        for scope in analysis
            .scopes
            .iter()
            .filter(|scope| Self::is_script_setup_component_import(analysis, scope))
        {
            for (name, binding) in scope.bindings() {
                if Self::is_component_binding(binding.binding_type) && is_pascal_case(name) {
                    push_component_candidate(
                        &mut candidates,
                        name,
                        name,
                        true,
                        scope.parent(),
                        analysis.binding_spans.get(name).copied(),
                    );
                }
            }
        }

        for registration in &analysis.component_registrations {
            push_component_candidate(
                &mut candidates,
                registration.name.as_str(),
                registration.local_name.as_str(),
                false,
                None,
                script_source
                    .and_then(|source| registration_key_span(source, registration.name.as_str())),
            );
        }

        candidates.sort_unstable_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
        candidates
    }

    fn is_script_setup_component_import(analysis: &Croquis, scope: &Scope) -> bool {
        let ScopeData::ExternalModule(data) = scope.data() else {
            return false;
        };
        if data.is_type_only || !Self::is_component_import_source(data.source.as_str()) {
            return false;
        }
        scope
            .parent()
            .and_then(|id| analysis.scopes.get_scope(id))
            .is_some_and(|parent| parent.kind == ScopeKind::ScriptSetup)
    }

    fn component_name_matches(used: &str, registered: &str) -> bool {
        used == registered
            || vize_croquis::naming::names_match(used, registered)
            || to_pascal_case(used).as_str() == registered
    }

    /// Preserve a negative conclusion when HTML recovery withholds Croquis.
    ///
    /// This fallback accepts only clean Options API script parses and compares
    /// registrations with authored opening-tag names. Raw markup can make a
    /// component look used (comments and strings are deliberately tolerated),
    /// but it cannot make an authored tag disappear and cause a false report.
    fn report_recovered_options_registrations(&self, ctx: &mut LintContext<'_>) {
        let reports = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let Some(script) = descriptor.script.as_ref() else {
                return;
            };
            let Some(template) = descriptor.template.as_ref() else {
                return;
            };
            let Some(parsed) = ctx.sfc_scripts().and_then(|scripts| scripts.script()) else {
                return;
            };
            if parsed.panicked
                || !parsed.diagnostics.is_empty()
                || raw_template_may_resolve_dynamic_component(template.content.as_ref())
            {
                return;
            }

            let used = authored_opening_tag_names(template.content.as_ref());
            let jsx = ctx
                .is_rule_enabled("vue/jsx-uses-vars")
                .then(|| jsx_referenced_names(parsed));
            vize_croquis::script_parser::parse_script(script.content.as_ref())
                .component_registrations
                .into_iter()
                .filter_map(|registration| {
                    let name = registration.name.as_str();
                    if self.should_ignore(name)
                        || used
                            .iter()
                            .any(|used| Self::component_name_matches(used.as_str(), name))
                        || jsx.as_ref().is_some_and(|names| {
                            names.contains(registration.local_name.as_str()) || names.contains(name)
                        })
                    {
                        return None;
                    }
                    let (start, end) = registration_key_span(script.content.as_ref(), name)?;
                    Some((registration.name, start, end))
                })
                .collect::<Vec<_>>()
        };

        let Some(script) = ctx
            .sfc_descriptor()
            .and_then(|descriptor| descriptor.script.as_ref())
        else {
            return;
        };
        let offset = script.loc.start as u32;
        for (name, start, end) in reports {
            ctx.report_in_sfc(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    cstr!(
                        "Component '{}' is registered but never used in template",
                        name
                    ),
                    offset + start,
                    offset + end,
                )
                .with_help("Remove the unused import or use the component in your template"),
            );
        }
    }
}

impl Rule for NoUnusedComponents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            self.report_recovered_options_registrations(ctx);
            return;
        }
        if has_dynamic_is_binding(&root.children) {
            return;
        }

        // Split-script Croquis coordinates append `<script setup>` after the
        // regular script. The fallback parser below reads setup content alone,
        // so its import spans must use that same local coordinate space.
        let setup_script_offset = ctx
            .sfc_descriptor()
            .and_then(|descriptor| {
                descriptor
                    .script
                    .as_ref()
                    .zip(descriptor.script_setup.as_ref())
            })
            .map_or(0, |(script, _)| script.content.len() as u32 + 1);

        // Collect template-unused components first (to avoid borrow conflicts)
        let (template_unused_components, import_statement_ranges): (
            Vec<ComponentCandidate>,
            Vec<(u32, u32)>,
        ) = {
            let Some(analysis) = ctx.analysis() else {
                return;
            };

            let script_source = ctx
                .sfc_descriptor()
                .and_then(|descriptor| descriptor.script.as_ref())
                .map(|script| script.content.as_ref());
            let registered_components = Self::component_candidates(analysis, script_source);

            let import_statement_ranges = analysis
                .import_statements
                .iter()
                .filter_map(|import| {
                    Some((
                        import.start.checked_sub(setup_script_offset)?,
                        import.end.checked_sub(setup_script_offset)?,
                    ))
                })
                .collect();

            let template_unused_components = registered_components
                .into_iter()
                .filter(|component| {
                    let name = component.name.as_str();
                    if self.should_ignore(name) {
                        return false;
                    }

                    if component.is_script_setup_import
                        && ctx.sfc_descriptor().is_some()
                        && component.setup_scope.zip(component.span).is_some_and(
                            |(scope, (start, end))| {
                                analysis.has_authored_template_binding_read(
                                    scope,
                                    component.local_name.as_str(),
                                    start,
                                    end,
                                )
                            },
                        )
                    {
                        return false;
                    }

                    // Check if used in template (case-insensitive matching for kebab-case)
                    !vize_croquis::facts::used_component_name_list(analysis)
                        .iter()
                        .any(|used| Self::component_name_matches(used.as_str(), name))
                })
                .collect();

            (template_unused_components, import_statement_ranges)
        };

        let script_setup_candidate_names: Vec<_> = template_unused_components
            .iter()
            .filter(|component| component.is_script_setup_import)
            .map(|component| component.local_name.clone())
            .collect();
        let script_used_components = script_setup_component_import_references(
            ctx,
            &script_setup_candidate_names,
            &import_statement_ranges,
        );
        let credit_jsx = ctx.is_rule_enabled("vue/jsx-uses-vars");
        let setup_jsx = credit_jsx.then(|| {
            ctx.sfc_scripts()
                .and_then(|scripts| scripts.script_setup())
                .map(jsx_referenced_names)
                .unwrap_or_default()
        });
        let script_jsx = credit_jsx.then(|| {
            ctx.sfc_scripts()
                .and_then(|scripts| scripts.script())
                .map(jsx_referenced_names)
                .unwrap_or_default()
        });
        let unused_components: Vec<_> = template_unused_components
            .into_iter()
            .filter(|component| {
                if component.is_script_setup_import {
                    if script_used_components.contains(component.local_name.as_str()) {
                        return false;
                    }
                    return !setup_jsx
                        .as_ref()
                        .is_some_and(|names| names.contains(component.local_name.as_str()));
                }
                !script_jsx.as_ref().is_some_and(|names| {
                    names.contains(component.local_name.as_str())
                        || names.contains(component.name.as_str())
                })
            })
            .collect();

        // Report unused components
        for component in unused_components {
            let name = component.name.as_str();
            let Some((start, end)) = component.span else {
                continue;
            };
            let mut diagnostic = crate::diagnostic::LintDiagnostic::warn(
                ctx.current_rule,
                cstr!(
                    "Component '{}' is registered but never used in template",
                    name
                ),
                start,
                end,
            )
            .with_help("Remove the unused import or use the component in your template");
            if component.is_script_setup_import {
                ctx.report_in_script(diagnostic);
            } else if let Some(script) = ctx
                .sfc_descriptor()
                .and_then(|descriptor| descriptor.script.as_ref())
            {
                let offset = script.loc.start as u32;
                diagnostic.start += offset;
                diagnostic.end += offset;
                ctx.report_in_sfc(diagnostic);
            }
        }
    }
}

fn raw_template_may_resolve_dynamic_component(template: &str) -> bool {
    // Stay deliberately conservative: whitespace around `=` and malformed
    // dynamic arguments are common precisely in the recovery lane. A false
    // match only withholds an unused conclusion; a missed match could create a
    // false positive for a component selected at runtime.
    template.contains(":is") || template.contains("v-bind") || template.contains(":[")
}

fn authored_opening_tag_names(template: &str) -> FxHashSet<CompactString> {
    let bytes = template.as_bytes();
    let mut names = FxHashSet::default();
    let mut cursor = 0;
    while let Some(relative) = template.get(cursor..).and_then(|rest| rest.find('<')) {
        let start = cursor + relative + 1;
        cursor = start;
        if matches!(bytes.get(start), Some(b'/' | b'!' | b'?')) {
            continue;
        }
        let mut end = start;
        while bytes.get(end).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.')
        }) {
            end += 1;
        }
        if end > start
            && let Some(name) = template.get(start..end)
        {
            names.insert(name.to_compact_string());
        }
        cursor = end.max(cursor + 1);
    }
    names
}

fn registration_key_span(source: &str, name: &str) -> Option<(u32, u32)> {
    let from = source.find("components").unwrap_or(0);
    let property_key = |start: usize| {
        source
            .get(start..)
            .unwrap_or_default()
            .match_indices(name)
            .find_map(|(relative, _)| {
                let at = start + relative;
                let prefix = source.get(..at).unwrap_or_default();
                let previous = prefix.trim_end().chars().next_back();
                let next = source
                    .get(at + name.len()..)
                    .unwrap_or_default()
                    .trim_start()
                    .chars()
                    .next();
                let starts_line = prefix
                    .rsplit_once('\n')
                    .is_some_and(|(_, line)| line.trim().is_empty());
                ((starts_line || matches!(previous, Some('{' | ',' | '\'' | '"')))
                    && matches!(next, Some(':' | ',' | '}' | '\'' | '"')))
                .then_some(at)
            })
    };
    let at = property_key(from).or_else(|| property_key(0)).or_else(|| {
        source.match_indices(name).find_map(|(at, _)| {
            let before = source.get(..at).unwrap_or_default().chars().next_back();
            let after = source
                .get(at + name.len()..)
                .unwrap_or_default()
                .chars()
                .next();
            let ident = |ch: char| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$';
            (!before.is_some_and(ident) && !after.is_some_and(ident)).then_some(at)
        })
    })?;
    Some((at as u32, (at + name.len()) as u32))
}

fn push_component_candidate(
    candidates: &mut Vec<ComponentCandidate>,
    name: &str,
    local_name: &str,
    is_script_setup_import: bool,
    setup_scope: Option<ScopeId>,
    span: Option<(u32, u32)>,
) {
    if candidates
        .iter()
        .any(|candidate| candidate.name.as_str() == name)
    {
        return;
    }
    candidates.push(ComponentCandidate {
        name: name.to_compact_string(),
        local_name: local_name.to_compact_string(),
        is_script_setup_import,
        setup_scope,
        span,
    });
}
