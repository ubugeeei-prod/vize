//! Template ref and Options API setup-return unwrapping.

use vize_carton::{FxHashSet, String, append};
use vize_croquis::{BindingType, Croquis, OptionGroup};

use super::options_api_support::is_safe_value_identifier;
use super::spans::is_local_setup_binding;

const REF_UNWRAP_HELPER: &str = "    type __U<T> = T extends import('vue').Ref ? T['value'] : T;\n";

pub(super) struct TemplateRefUnwraps {
    setup_bindings: Vec<String>,
    options_api_setup_bindings: Vec<String>,
}

impl TemplateRefUnwraps {
    pub(super) fn collect(
        summary: &Croquis,
        options_api: bool,
        template_referenced_names: Option<&FxHashSet<String>>,
    ) -> Self {
        let mut options_api_setup_bindings =
            collect_options_api_setup_bindings(summary, options_api);
        if let Some(template_referenced_names) = template_referenced_names {
            options_api_setup_bindings
                .retain(|name| template_referenced_names.contains(name.as_str()));
        }
        let options_api_setup_binding_names: FxHashSet<&str> = options_api_setup_bindings
            .iter()
            .map(|name| name.as_str())
            .collect();

        let mut setup_bindings: Vec<String> = summary
            .bindings
            .bindings
            .iter()
            .filter(|(name, _)| {
                template_referenced_names
                    .is_none_or(|referenced| referenced.contains(name.as_str()))
            })
            .filter(|(name, _)| !options_api_setup_binding_names.contains(name.as_str()))
            .filter(|(name, binding_type)| {
                summary.reactivity.needs_value_access(name.as_str())
                    || matches!(binding_type, BindingType::SetupMaybeRef)
                        && is_local_setup_binding(summary, name.as_str())
            })
            .map(|(name, _)| String::from(name.as_str()))
            .collect();
        setup_bindings.sort_unstable();

        Self {
            setup_bindings,
            options_api_setup_bindings,
        }
    }

    pub(super) fn emit_type_captures(&self, mut ts: &mut String) {
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

    pub(super) fn emit_template_variables(&self, mut ts: &mut String) {
        if self.setup_bindings.is_empty() && self.options_api_setup_bindings.is_empty() {
            return;
        }

        ts.push_str("    // Auto-unwrap Vue refs in template scope\n");
        ts.push_str(REF_UNWRAP_HELPER);
        for name in &self.setup_bindings {
            append!(ts, "    var {name}: __U<__R_{name}> = undefined as any;\n");
        }
        for name in &self.options_api_setup_bindings {
            append!(ts, "    var {name}: __U<__R_{name}> = undefined as any;\n");
        }
    }
}

fn collect_options_api_setup_bindings(summary: &Croquis, options_api: bool) -> Vec<String> {
    if !options_api || summary.bindings.is_script_setup {
        return Vec::new();
    }

    let Some(descriptor) = summary.options_descriptor.as_ref() else {
        return Vec::new();
    };

    let mut names: Vec<String> = descriptor
        .members_in(OptionGroup::Setup)
        .map(|member| member.name.as_str())
        .filter(|name| {
            is_safe_value_identifier(name)
                && matches!(
                    summary.bindings.get(name),
                    Some(BindingType::SetupMaybeRef | BindingType::SetupRef)
                )
        })
        .map(String::from)
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}
