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
use vize_croquis::{BindingType, Croquis, ScopeData, ScopeId};

use super::expressions::{BindingRewrite, PreparedInstanceRewrite, prepare_instance_rewrite};
use super::types::{VirtualTsOptions, VizeMapping};

#[derive(Clone, Copy, Debug)]
enum Receiver {
    Props,
    OptionsInstance,
}

type RewriteCache = FxHashMap<ScopeId, FxHashMap<String, RewriteEntry>>;

#[derive(Debug)]
enum RewriteEntry {
    Prepared(PreparedInstanceRewrite),
    Ready(Option<BindingRewrite>),
}

/// Immutable admission and relative rewrite results for one generated file.
/// Scope-specific views borrow this value; no emission offset enters the cache.
#[derive(Debug)]
struct InstanceAccess<'a> {
    summary: &'a Croquis,
    options: &'a VirtualTsOptions,
    setup_spread_bindings: &'a [String],
    infer_dollar_el: bool,
    /// Chosen from actual normalized semantic facts before any emission.
    receiver: String,
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
                receiver: String::from("__ctx"),
                cache: RefCell::default(),
                diagnostic_prefixes: RefCell::default(),
            }));
            self.prepare_instance_context();
        }
        self
    }

    /// Prepare source-relative semantic plans before selecting a receiver.
    /// Ordinary files retain `__ctx`; only a proven collision needs the full
    /// authored-name reservation pass and an alias of that same typed value.
    fn prepare_instance_context(&mut self) {
        let Some(instance) = self.instance.as_ref().map(InstanceStorage::get) else {
            return;
        };
        let summary = instance.summary;
        self.prepare_expression_plans(summary, true);
        let has_references = instance.cache.borrow().values().any(|scope| {
            scope.values().any(|entry| matches!(entry, RewriteEntry::Prepared(plan) if plan.has_instance_references()))
        });
        let collides = instance.cache.borrow().values().any(|scope| {
            scope.values().any(|entry| matches!(entry, RewriteEntry::Prepared(plan) if plan.receiver_is_shadowed()))
        });
        let receiver = if has_references && collides {
            // A missing candidate-name read may be in a non-dollar expression.
            // Reserve its semantic identity too; never let the alias make that
            // previously invalid authored program clean.
            self.prepare_expression_plans(summary, false);
            let mut names = FxHashSet::default();
            super::script_facts::with_bindings(summary, |bindings| {
                names.extend(bindings.typed().map(|(name, _)| String::from(name)));
            });
            names.extend(
                super::script_facts::undefined_refs(summary)
                    .into_iter()
                    .map(|row| String::from(row.name.as_str())),
            );
            names.extend(self.names.keys().cloned());
            for scope in summary.scopes.iter() {
                names.extend(scope.bindings().map(|(name, _)| String::from(name)));
            }
            for scope in instance.cache.borrow().values() {
                for entry in scope.values() {
                    if let RewriteEntry::Prepared(plan) = entry {
                        names.extend(plan.normalized_names().iter().cloned());
                    }
                }
            }
            names.extend(instance.setup_spread_bindings.iter().cloned());
            names.extend(
                summary
                    .type_exports
                    .iter()
                    .map(|row| String::from(row.name.as_str())),
            );
            names.extend(
                vize_croquis::facts::used_component_name_list(summary)
                    .into_iter()
                    .map(|name| String::from(name.as_str())),
            );
            names.extend(
                instance
                    .options
                    .template_globals
                    .iter()
                    .map(|row| row.name.clone()),
            );
            names.extend(instance.options.css_modules.iter().cloned());
            names.extend(instance.options.external_template_bindings.iter().cloned());
            let mut candidate = String::from("__vize_template_instance");
            while names.contains(candidate.as_str())
                || instance
                    .options
                    .has_auto_import_binding_name(candidate.as_str())
            {
                candidate.push('_');
            }
            candidate
        } else {
            String::from("__ctx")
        };
        if let Some(InstanceStorage::Owned(instance)) = self.instance.as_mut() {
            instance.receiver = receiver;
        }
        self.materialize_prepared_rewrites();
    }

    fn prepare_expression_plans(&self, summary: &Croquis, dollar_only: bool) {
        for expression in &summary.template_expressions {
            self.prepare_expression_once(
                expression.content.as_str().trim(),
                expression.scope_id,
                dollar_only,
            );
            if let Some(guard) = &expression.vif_guard {
                self.prepare_expression_once(
                    guard.as_str().trim(),
                    expression.scope_id,
                    dollar_only,
                );
            }
        }
        for scope in summary.scopes.iter() {
            if let ScopeData::VFor(data) = scope.data() {
                self.prepare_expression_once(
                    data.source.as_str().trim(),
                    scope.parent().unwrap_or(scope.id),
                    dollar_only,
                );
            }
        }
    }

    fn prepare_expression_once(&self, expression: &str, scope: ScopeId, dollar_only: bool) {
        if expression.is_empty() || (dollar_only && !expression.contains('$')) {
            return;
        }
        let Some(instance) = self.instance.as_ref().map(InstanceStorage::get) else {
            return;
        };
        if instance
            .cache
            .borrow()
            .get(&scope)
            .is_some_and(|rows| rows.contains_key(expression))
        {
            return;
        }
        let view = self.for_scope(scope);
        let plan = prepare_instance_rewrite(expression, &view);
        vize_carton::profiler::global_profiler()
            .record_counter("canon.instance_global_rewrite.cache_misses", 1);
        vize_carton::profiler::global_profiler()
            .record_counter("canon.instance_global_rewrite.cache_entries", 1);
        instance
            .cache
            .borrow_mut()
            .entry(scope)
            .or_default()
            .insert(String::from(expression), RewriteEntry::Prepared(plan));
    }

    fn materialize_prepared_rewrites(&self) {
        let Some(instance) = self.instance.as_ref().map(InstanceStorage::get) else {
            return;
        };
        // Materialization reads immutable admission/receiver facts only. It
        // neither reparses nor enters this cache, so each relative plan moves
        // directly into its final value without shared allocations or copying.
        for (scope, entries) in instance.cache.borrow_mut().iter_mut() {
            let view = self.for_scope(*scope);
            for (expression, entry) in entries {
                if let RewriteEntry::Prepared(plan) =
                    std::mem::replace(entry, RewriteEntry::Ready(None))
                {
                    *entry = RewriteEntry::Ready(plan.materialize(expression.as_str(), &view));
                }
            }
        }
    }

    pub(crate) fn emit_instance_receiver_alias(&self, output: &mut String) {
        if let Some(instance) = self.instance.as_ref().map(InstanceStorage::get)
            && instance.receiver != "__ctx"
        {
            vize_carton::append!(
                *output,
                "  const {name} = __ctx;\n",
                name = instance.receiver
            );
        }
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
                .insert(String::from(expression), RewriteEntry::Ready(result));
        }
        Ref::filter_map(instance.cache.borrow(), |cache| {
            match cache.get(&self.scope_id)?.get(expression)? {
                RewriteEntry::Ready(value) => value.as_ref(),
                RewriteEntry::Prepared(_) => None,
            }
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

    pub(crate) fn receiver(&self, name: &str) -> Option<&str> {
        if self.is_instance_global(name) {
            return self
                .instance
                .as_ref()
                .map(|value| value.get().receiver.as_str());
        }
        self.names.get(name).map(|receiver| match receiver {
            Receiver::Props => "props",
            Receiver::OptionsInstance => "__vize_options_instance",
        })
    }
}
