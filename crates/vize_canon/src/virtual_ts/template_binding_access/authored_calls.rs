//! Authored-instance call receivers share the existing semantic rewrite plan.
use super::*;
use crate::virtual_ts::scope::{
    ScopeGenerationOptions, is_declared_template_context_name,
    is_strict_template_context_candidate, script_top_level_binding_names,
};

impl<'a> TemplateBindingAccess<'a> {
    pub(crate) fn for_template(
        summary: &'a Croquis,
        source: &ScopeGenerationOptions<'a, '_>,
    ) -> Self {
        let authored = source.options_api
            && source.has_default_alias
            && !source.legacy_vue2
            && !summary
                .scopes
                .iter()
                .any(|scope| scope.kind == vize_croquis::ScopeKind::ScriptSetup);
        let result = Self::collect(summary, source.options_api).with_instance_globals(
            summary,
            source.virtual_ts_options,
            source.setup_spread_bindings,
            source.check_options.infer_template_dollar_el,
            source.check_options.check_template_bindings
                && !source.legacy_vue2
                && !source.virtual_ts_options.strict_instance_globals
                && source
                    .template_ast
                    .is_some_and(|root| root.source.contains('$'))
                && !authored,
        );
        if !authored
            || !source.check_options.check_template_bindings
            || source.template_ast.is_none()
        {
            return result;
        }
        let script_names = match source.script_content {
            Some(script) => match script_top_level_binding_names(script) {
                Some(names) => names,
                None => return result,
            },
            None => FxHashSet::default(),
        };
        let eligible: FxHashSet<String> = super::super::script_facts::undefined_refs(summary)
            .into_iter()
            .map(|row| String::from(row.name.as_str()))
            .filter(|name| {
                is_strict_template_context_candidate(name)
                    && !super::super::scope::is_template_instance_global_name(name)
                    && !result.is_known(name)
                    && !script_names.contains(name)
                    && !is_declared_template_context_name(name, source.virtual_ts_options)
                    && !source.keyed_prop_bindings.contains(name)
            })
            .collect();
        if eligible.is_empty() {
            return result;
        }
        let mut result = result;
        result.instance = Some(InstanceStorage::Owned(InstanceAccess {
            owner: InstanceOwner::AuthoredCalls {
                eligible,
                script_names,
            },
            summary,
            options: source.virtual_ts_options,
            setup_spread_bindings: source.setup_spread_bindings,
            infer_dollar_el: source.check_options.infer_template_dollar_el,
            receiver: String::from("__ctx"),
            cache: RefCell::default(),
            diagnostic_prefixes: RefCell::default(),
        }));
        result.prepare_instance_context();
        result
    }

    pub(crate) fn has_authored_instance_calls(&self) -> bool {
        self.instance
            .as_ref()
            .is_some_and(|value| matches!(value.get().owner, InstanceOwner::AuthoredCalls { .. }))
    }

    pub(crate) fn is_authored_instance_callee(&self, name: &str) -> bool {
        self.instance
            .as_ref()
            .is_some_and(|value| match &value.get().owner {
                InstanceOwner::AuthoredCalls { eligible, .. } => eligible.contains(name),
                InstanceOwner::PublicGlobals => false,
            })
    }

    pub(crate) fn authored_callee_receiver(&self) -> Option<&str> {
        self.instance
            .as_ref()
            .filter(|value| matches!(value.get().owner, InstanceOwner::AuthoredCalls { .. }))
            .map(|value| value.get().receiver.as_str())
    }
}
