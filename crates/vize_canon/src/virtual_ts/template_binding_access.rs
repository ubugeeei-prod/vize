//! Property receivers for template names which cannot be declared as locals.
//!
//! Props and Options API members have different owners. Keep that ownership in
//! one projection input so guards, loops, event handlers and prop checks resolve
//! reserved names through the same typed object.

use super::{helpers::is_reserved_identifier, props::collect_template_prop_names};
use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
};
use vize_carton::{FxHashMap, FxHashSet, String};
use vize_croquis::{BindingType, Croquis, ScopeId};

use super::expressions::BindingRewrite;
use super::types::{VirtualTsOptions, VizeMapping};

#[derive(Clone, Copy, Debug)]
enum Receiver {
    Props,
    OptionsInstance,
}

type RewriteCache = FxHashMap<ScopeId, FxHashMap<String, Option<BindingRewrite>>>;

/// Immutable admission and relative rewrite results for one generated file.
/// Scope-specific views borrow this value; no emission offset enters the cache.
#[derive(Debug)]
struct InstanceAccess<'a> {
    summary: &'a Croquis,
    options: &'a VirtualTsOptions,
    setup_spread_bindings: &'a [String],
    infer_dollar_el: bool,
    cache: RefCell<RewriteCache>,
    /// Actual generated positions belong to emission, independently of the pure cache.
    diagnostic_prefixes: RefCell<Vec<VizeMapping>>,
}

#[derive(Debug)]
enum InstanceStorage<'a> {
    Owned(InstanceAccess<'a>),
    Borrowed(&'a InstanceAccess<'a>),
}

impl InstanceStorage<'_> {
    fn get(&self) -> &InstanceAccess<'_> {
        match self {
            Self::Owned(value) => value,
            Self::Borrowed(value) => value,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TemplateBindingAccess<'a> {
    names: Cow<'a, FxHashMap<String, Receiver>>,
    instance: Option<InstanceStorage<'a>>,
    scope_id: ScopeId,
}

impl Default for TemplateBindingAccess<'_> {
    fn default() -> Self {
        Self {
            names: Cow::Owned(FxHashMap::default()),
            instance: None,
            scope_id: ScopeId::ROOT,
        }
    }
}

impl<'a> TemplateBindingAccess<'a> {
    pub(crate) fn collect(summary: &Croquis, options_api: bool) -> Self {
        let mut result = Self::from_props(collect_template_prop_names(summary));
        if options_api {
            super::script_facts::with_bindings(summary, |bindings| {
                let script_setup = bindings.is_script_setup();
                for (name, binding) in bindings.typed() {
                    if (matches!(binding, BindingType::Data | BindingType::Options)
                        || (binding == BindingType::Props && !script_setup))
                        && is_reserved_identifier(name)
                    {
                        result
                            .names
                            .to_mut()
                            .insert(name.into(), Receiver::OptionsInstance);
                    }
                }
            });
        }
        result
    }

    pub(crate) fn from_props(names: FxHashSet<String>) -> Self {
        Self {
            names: Cow::Owned(
                names
                    .into_iter()
                    .map(|name| (name, Receiver::Props))
                    .collect(),
            ),
            ..Self::default()
        }
    }

    pub(crate) fn with_instance_globals(
        mut self,
        summary: &'a Croquis,
        options: &'a VirtualTsOptions,
        setup_spread_bindings: &'a [String],
        infer_dollar_el: bool,
        enabled: bool,
    ) -> Self {
        if enabled {
            self.instance = Some(InstanceStorage::Owned(InstanceAccess {
                summary,
                options,
                setup_spread_bindings,
                infer_dollar_el,
                cache: RefCell::default(),
                diagnostic_prefixes: RefCell::default(),
            }));
        }
        self
    }

    pub(crate) fn for_scope(&self, scope_id: ScopeId) -> TemplateBindingAccess<'_> {
        TemplateBindingAccess {
            names: Cow::Borrowed(self.names.as_ref()),
            instance: self
                .instance
                .as_ref()
                .map(|instance| InstanceStorage::Borrowed(instance.get())),
            scope_id,
        }
    }

    pub(crate) fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    pub(crate) fn instance_summary(&self) -> Option<&Croquis> {
        self.instance
            .as_ref()
            .map(|instance| instance.get().summary)
    }

    pub(crate) fn has_instance_globals(&self) -> bool {
        self.instance.is_some()
    }

    pub(crate) fn is_instance_global(&self, name: &str) -> bool {
        let Some(instance) = self.instance.as_ref().map(InstanceStorage::get) else {
            return false;
        };
        super::scope::is_template_instance_global_name(name)
            && name != "$props"
            && !(name == "$el" && instance.infer_dollar_el)
            && !super::scope::is_declared_template_context_name(name, instance.options)
            && !super::script_facts::contains_binding(instance.summary, name)
            && !instance
                .setup_spread_bindings
                .iter()
                .any(|binding| binding.as_str() == name)
            && !instance
                .summary
                .type_exports
                .iter()
                .any(|export| export.name.as_str() == name)
    }

    /// Parse at most once per expression and actual template scope in this
    /// immutable admission context. The caller still maps every emitted copy.
    pub(crate) fn cached_instance_rewrite(
        &self,
        expression: &str,
        build: impl FnOnce() -> Option<BindingRewrite>,
    ) -> Option<Ref<'_, BindingRewrite>> {
        let instance = self.instance.as_ref()?.get();
        let cached = instance
            .cache
            .borrow()
            .get(&self.scope_id)
            .is_some_and(|scope| scope.contains_key(expression));
        if cached {
            vize_carton::profiler::global_profiler()
                .record_counter("canon.instance_global_rewrite.cache_hits", 1);
        } else {
            vize_carton::profiler::global_profiler()
                .record_counter("canon.instance_global_rewrite.cache_misses", 1);
            let result = build();
            vize_carton::profiler::global_profiler()
                .record_counter("canon.instance_global_rewrite.cache_entries", 1);
            instance
                .cache
                .borrow_mut()
                .entry(self.scope_id)
                .or_default()
                .insert(String::from(expression), result);
        }
        Ref::filter_map(instance.cache.borrow(), |cache| {
            cache.get(&self.scope_id)?.get(expression)?.as_ref()
        })
        .ok()
    }

    pub(crate) fn record_instance_diagnostic_prefix(&self, mapping: VizeMapping) {
        if let Some(instance) = &self.instance {
            instance
                .get()
                .diagnostic_prefixes
                .borrow_mut()
                .push(mapping);
            vize_carton::profiler::global_profiler()
                .record_counter("canon.instance_global_rewrite.diagnostic_prefixes", 1);
        }
    }

    pub(crate) fn append_instance_diagnostic_prefixes(&self, output: &mut Vec<VizeMapping>) {
        if let Some(instance) = &self.instance {
            output.append(&mut instance.get().diagnostic_prefixes.borrow_mut());
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.names.is_empty() && self.instance.is_none()
    }

    pub(crate) fn has_reserved_bindings(&self) -> bool {
        !self.names.is_empty()
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name) || self.is_instance_global(name)
    }

    pub(crate) fn receiver(&self, name: &str) -> Option<&'static str> {
        if self.is_instance_global(name) {
            return Some("__ctx");
        }
        self.names.get(name).map(|receiver| match receiver {
            Receiver::Props => "props",
            Receiver::OptionsInstance => "__vize_options_instance",
        })
    }
}
