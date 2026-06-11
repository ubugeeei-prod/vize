//! Result and option types produced by the script parser.
//!
//! Houses [`ScriptParseResult`] (the full script-analysis payload),
//! [`ScriptParserOptions`], and the small metadata enums/structs that back
//! plain-value and runtime-object tracking.

use crate::croquis::{BindingMetadata, ComponentRegistration, ComponentShape, Croquis};
use crate::croquis::{ImportStatementInfo, InvalidExport, ReExportInfo, TypeExport};
use crate::macros::{EmitDefinition, MacroTracker, PropDefinition};
use crate::provide::ProvideInjectTracker;
use crate::race::RaceConditionTracker;
use crate::reactivity::ReactivityTracker;
use crate::scope::ScopeChain;
use crate::setup_context::SetupContextTracker;
use vize_carton::{CompactString, FxHashMap, FxHashSet};

/// Origin of a local binding that already carries a plain, non-reactive value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReactiveValueOrigin {
    PropsDestructure {
        prop_name: CompactString,
    },
    ReactiveProperty {
        source_name: CompactString,
        prop_name: CompactString,
    },
    RefValue {
        source_name: CompactString,
    },
    FunctionArgument {
        source_name: CompactString,
        callee_name: CompactString,
    },
    GetterCall {
        context_name: CompactString,
        getter_name: CompactString,
        source_name: CompactString,
    },
    PlainAlias {
        source_name: CompactString,
    },
}

/// A returned context whose methods are backed by getter arguments.
#[derive(Debug, Clone, Default)]
pub(crate) struct ReactiveGetterContext {
    pub callee_name: CompactString,
    pub getters: FxHashMap<CompactString, CompactString>,
}

/// Static metadata extracted from a top-level runtime object literal.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeObjectLiteral {
    pub props: Vec<PropDefinition>,
    pub emits: Vec<EmitDefinition>,
}

/// Result of parsing a script setup block
#[derive(Debug, Default)]
pub struct ScriptParseResult {
    pub bindings: BindingMetadata,
    pub macros: MacroTracker,
    pub reactivity: ReactivityTracker,
    pub race_conditions: RaceConditionTracker,
    pub type_exports: Vec<TypeExport>,
    pub invalid_exports: Vec<InvalidExport>,
    /// Scope chain for tracking nested JavaScript scopes
    pub scopes: ScopeChain,
    /// Provide/Inject tracking
    pub provide_inject: ProvideInjectTracker,
    /// Track inject variable names for indirect destructure detection
    pub(crate) inject_var_names: FxHashSet<CompactString>,
    /// Track aliases for inject function (e.g., const a = inject; a('key'))
    pub(crate) inject_aliases: FxHashSet<CompactString>,
    /// Track aliases for provide function (e.g., const p = provide; p('key', val))
    pub(crate) provide_aliases: FxHashSet<CompactString>,
    /// Track aliases for reactivity APIs (e.g., const r = ref; r(0))
    /// Maps alias name to the original function name
    pub(crate) reactivity_aliases: FxHashMap<CompactString, CompactString>,
    /// Bindings that are known plain snapshots of reactive values.
    pub(crate) reactive_value_origins: FxHashMap<CompactString, ReactiveValueOrigin>,
    /// Call results that were constructed from getter arguments.
    pub(crate) reactive_getter_contexts: FxHashMap<CompactString, ReactiveGetterContext>,
    /// Setup context violation tracking
    pub setup_context: SetupContextTracker,
    /// Flag to track if we're in a non-setup script context
    pub(crate) is_non_setup_script: bool,
    /// Import statement spans in script content
    pub import_statements: Vec<ImportStatementInfo>,
    /// Re-export statement spans (`export { ... } from "..."`)
    pub re_exports: Vec<ReExportInfo>,
    /// Components registered through Options API `components`.
    pub component_registrations: Vec<ComponentRegistration>,
    /// API shape of the component's default export (e.g. class component).
    pub component_shape: ComponentShape,
    /// Definition spans for bindings (name -> (start, end) offset in script)
    pub binding_spans: FxHashMap<CompactString, (u32, u32)>,
    /// Value import source by local binding name.
    pub import_sources: FxHashMap<CompactString, CompactString>,
    /// Names referenced via `typeof X` in the body of each `type_exports`
    /// entry, indexed in parallel with `type_exports`. Used by
    /// `resolve_type_export_hoisting` to keep types adjacent to the
    /// setup-scope values they depend on. Pushed to in lockstep with
    /// `type_exports` via `record_type_export`.
    pub(crate) type_export_typeof_refs: Vec<FxHashSet<CompactString>>,
    /// Static runtime object literal metadata available to macro spread args.
    pub(crate) runtime_object_literals: FxHashMap<CompactString, RuntimeObjectLiteral>,
}

/// Options for plain script parsing.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScriptParserOptions {
    /// Resolve Options API template bindings (`data`/`computed`/`methods`/
    /// `inject`/`setup`/`props`). This is officially supported in Vue 3 and is
    /// an opt-in for the standard build — it does **not** require the `legacy`
    /// feature.
    pub options_api: bool,
    /// Additionally treat the component as legacy Vue 2.7 / Nuxt 2: implies
    /// `options_api` binding resolution and adds Nuxt 2 template globals.
    pub legacy_vue2: bool,
}

impl ScriptParseResult {
    /// Record a `TypeExport` together with the `typeof` value-identifier
    /// references found in its body. Must be the only call site that pushes
    /// to `type_exports` so the two vectors stay in lockstep for
    /// `resolve_type_export_hoisting`.
    pub(crate) fn record_type_export(
        &mut self,
        export: TypeExport,
        typeof_refs: FxHashSet<CompactString>,
    ) {
        self.type_exports.push(export);
        self.type_export_typeof_refs.push(typeof_refs);
    }

    /// Demote `TypeExport::hoisted` to `false` for any type whose body
    /// references a setup-scope value binding via `typeof`. The virtual TS
    /// generator only lifts hoisted types to module scope, so demoted types
    /// stay inside the synthetic `__setup` function alongside the values
    /// they depend on — which is the only place TS can resolve them.
    ///
    /// Imports add value bindings at module scope, so a `typeof importedName`
    /// reference is left as hoisted: the import is visible from the module
    /// scope where the type lands.
    pub(crate) fn resolve_type_export_hoisting(&mut self) {
        if self.type_export_typeof_refs.is_empty() {
            return;
        }

        // A `typeof name` ref keeps a type hoisted only when `name` is a
        // module-scoped import. Rather than materialize the full set of
        // imported binding names up front (O(imports × bindings)), test each
        // referenced name's declaration span against the import ranges on
        // demand — `refs` is tiny, so this is O(refs × imports).
        for idx in 0..self.type_export_typeof_refs.len() {
            let touches_setup_value = self.type_export_typeof_refs[idx].iter().any(|name| {
                let key = name.as_str();
                self.bindings.bindings.contains_key(key)
                    && !self.binding_spans.get(key).is_some_and(|(start, end)| {
                        // Bindings whose declaration site falls inside an import
                        // statement are module-scoped imports, not setup values.
                        self.import_statements
                            .iter()
                            .any(|imp| *start >= imp.start && *end <= imp.end)
                    })
            });
            if touches_setup_value && let Some(te) = self.type_exports.get_mut(idx) {
                te.hoisted = false;
            }
        }
    }

    /// Apply script analysis fields to an existing SFC analysis summary.
    ///
    /// This keeps script parsing as the single owner of script-scoped data while
    /// allowing callers to add template analysis before or after the script pass.
    pub fn apply_to_croquis(self, summary: &mut Croquis) {
        summary.bindings = self.bindings;
        summary.macros = self.macros;
        summary.reactivity = self.reactivity;
        summary.race_conditions = self.race_conditions;
        summary.type_exports = self.type_exports;
        summary.invalid_exports = self.invalid_exports;
        summary.scopes = self.scopes;
        summary.provide_inject = self.provide_inject;
        summary.setup_context = self.setup_context;
        summary.import_statements = self.import_statements;
        summary.re_exports = self.re_exports;
        summary.component_registrations = self.component_registrations;
        summary.component_shape = self.component_shape;
        summary.binding_spans = self.binding_spans;
    }

    /// Convert script analysis into a `Croquis` summary.
    pub fn into_croquis(self) -> Croquis {
        let mut summary = Croquis::new();
        self.apply_to_croquis(&mut summary);
        summary
    }
}
