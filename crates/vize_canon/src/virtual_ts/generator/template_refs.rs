use vize_carton::config::VueVersion;
use vize_carton::{FxHashSet, String, append};
use vize_croquis::{BindingType, Croquis};

mod deferred_bindings;

use super::legacy_vue2::ref_unwrap_helper_for_template;
use super::spans::is_local_setup_binding;

pub(super) struct TemplateRefUnwraps {
    setup_bindings: Vec<String>,
    options_api_setup_bindings: Vec<String>,
    /// Setup bindings whose assignment is deferred past their declaration. See
    /// `deferred_bindings` for why they need the same shadowing treatment.
    deferred_bindings: Vec<String>,
}

impl TemplateRefUnwraps {
    pub(super) fn collect(
        summary: &Croquis,
        options_api: bool,
        template_referenced_names: Option<&FxHashSet<String>>,
        script_content: Option<&str>,
    ) -> Self {
        let options_api_setup_bindings =
            crate::options_api_setup_spread::collect_template_setup_bindings(
                summary,
                options_api,
                template_referenced_names,
                script_content,
            );
        let options_api_setup_binding_names: FxHashSet<&str> = options_api_setup_bindings
            .iter()
            .map(|name| name.as_str())
            .collect();

        let mut setup_bindings: Vec<String> = summary
            .bindings
            .bindings
            .iter()
            .filter(|(name, _)| {
                template_referenced_names.is_none_or(|referenced| {
                    referenced.contains(name.as_str())
                        || is_local_setup_binding(summary, name.as_str())
                })
            })
            .filter(|(name, _)| !options_api_setup_binding_names.contains(name.as_str()))
            .filter(|(name, binding_type)| {
                summary.reactivity.needs_value_access(name.as_str())
                    || matches!(binding_type, BindingType::SetupMaybeRef)
            })
            .map(|(name, _)| String::from(name.as_str()))
            .collect();
        setup_bindings.sort_unstable();

        // Shadowing a name twice would redeclare it with a conflicting type,
        // so the deferred set excludes everything already shadowed above.
        let deferred_bindings = deferred_bindings::collect_deferred_setup_bindings(
            summary,
            script_content,
            template_referenced_names,
            |name| {
                setup_bindings.iter().any(|shadowed| shadowed == name)
                    || options_api_setup_binding_names.contains(name)
            },
        );

        Self {
            setup_bindings,
            options_api_setup_bindings,
            deferred_bindings,
        }
    }

    pub(super) fn setup_spread_bindings(&self) -> &[String] {
        &self.options_api_setup_bindings
    }

    pub(super) fn emit_type_captures(&self, mut ts: &mut String) {
        deferred_bindings::emit_type_captures(ts, &self.deferred_bindings);
        if !self.setup_bindings.is_empty() {
            ts.push_str("  // Ref type captures (before template scope shadows them)\n");
            for name in &self.setup_bindings {
                append!(ts, "  type __R_{name} = typeof {name};\n");
            }
        }
        if !self.options_api_setup_bindings.is_empty() {
            ts.push_str(
                "  // Options API setup return type captures (before template scope shadows them)\n",
            );
            ts.push_str(
                "  type __VizeOptionsSetupBinding<K extends string> = typeof __default__ extends abstract new (...args: any) => infer __I ? K extends keyof __I ? __I[K] : any : any;\n",
            );
            for name in &self.options_api_setup_bindings {
                append!(
                    ts,
                    "  type __R_{name} = __VizeOptionsSetupBinding<\"{name}\">;\n"
                );
            }
        }
    }

    /// Shadow ref bindings with their unwrapped types. `var` allows
    /// reassignment, which Vue templates do to refs.
    ///
    /// Emits nothing at all when template scope has no setup binding to
    /// shadow, which is why the conditional types `__U` delegates to are
    /// declared alongside it rather than at module scope — see
    /// `legacy_vue2::MODERN_REF_UNWRAP_HELPER`.
    pub(super) fn emit_template_variables(
        &self,
        mut ts: &mut String,
        legacy_vue2: bool,
        dialect: VueVersion,
        has_generic_param: bool,
        hoist_shared_preamble: bool,
    ) {
        deferred_bindings::emit_template_variables(ts, &self.deferred_bindings);
        if self.setup_bindings.is_empty() && self.options_api_setup_bindings.is_empty() {
            return;
        }

        ts.push_str("    // Auto-unwrap Vue refs in template scope\n");
        ts.push_str(ref_unwrap_helper_for_template(
            legacy_vue2,
            dialect,
            has_generic_param,
            hoist_shared_preamble,
        ));
        for name in &self.setup_bindings {
            append!(ts, "    var {name}: __U<__R_{name}> = undefined as any;\n");
        }
        for name in &self.options_api_setup_bindings {
            append!(ts, "    var {name}: __U<__R_{name}> = undefined as any;\n");
        }
    }
}
