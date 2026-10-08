//! Result and option types produced by the script parser.
//!
//! Houses [`ScriptParseResult`] (the full script-analysis payload),
//! [`ScriptParserOptions`], and the small metadata enums/structs that back
//! plain-value and runtime-object tracking.

mod debug;
mod origins;
mod ref_value_declaration;
mod ref_values;

use crate::croquis::{BindingMetadata, ComponentRegistration, ComponentShape, Croquis};
use crate::croquis::{
    ImportStatementInfo, InvalidExport, OptionsDescriptor, ReExportInfo, TypeExport,
};
use crate::macros::{EmitDefinition, MacroTracker, PropDefinition};
use crate::provide::ProvideInjectTracker;
use crate::race::RaceConditionTracker;
use crate::reactivity::ReactivityTracker;
use crate::scope::{ScopeChain, ScopeId};
use crate::script_parser::typeof_refs::TypeDependencyRefs;
use crate::setup_context::SetupContextTracker;
use crate::types::TypeResolver;
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
    GetterCall {
        context_name: CompactString,
        getter_name: CompactString,
        source_name: CompactString,
    },
    PlainAlias {
        source_name: CompactString,
    },
}

/// Lexical identity of a ref initializer or a shadowing ordinary binding.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefValueSourceKind {
    Live,
    Raw,
    Other,
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
    pub emit_validator_signatures: FxHashMap<CompactString, Vec<CompactString>>,
    pub emit_validator_type_annotations: FxHashMap<CompactString, Vec<CompactString>>,
}

/// Result of parsing a script setup block
#[derive(Default)]
pub struct ScriptParseResult {
    pub bindings: BindingMetadata,
    /// Private demand-only facts, never serialized in Croquis.
    pub(crate) occurrence_capture: Option<Box<super::occurrences::ScriptOccurrenceCapture>>,
    /// Setup declarations with no resolved script read, when demanded.
    pub unused_bindings: Vec<CompactString>,
    pub macros: MacroTracker,
    pub reactivity: ReactivityTracker,
    pub race_conditions: RaceConditionTracker,
    /// Local `interface` / `type` definitions registered by name, so
    /// `defineProps<Name>()` and template-binding analysis can resolve the
    /// fields of a locally declared type through an OXC-backed AST walk
    /// rather than a raw-text scan.
    pub types: TypeResolver,
    /// Exact local type-member ranges retained by the existing AST walk.
    pub(crate) type_property_declarations:
        FxHashMap<CompactString, FxHashMap<CompactString, Option<(u32, u32)>>>,
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
    /// Private lexical provenance; the name map above keeps the debug contract.
    pub(crate) scoped_reactive_value_origins:
        FxHashMap<ScopeId, FxHashMap<CompactString, ReactiveValueOrigin>>,
    /// Ref value member writes that retain a proxy or point at a template node.
    pub(crate) ref_value_sources: FxHashMap<ScopeId, FxHashMap<CompactString, RefValueSourceKind>>,
    /// A snapshot keeps the identity of its source even in a shadowing closure.
    pub(crate) live_ref_value_origins: FxHashMap<ScopeId, FxHashSet<CompactString>>,
    /// Call results that were constructed from getter arguments.
    pub(crate) reactive_getter_contexts: FxHashMap<CompactString, ReactiveGetterContext>,
    /// Setup context violation tracking, plus script browser-global reads.
    pub setup_context: SetupContextTracker,
    /// Flag to track if we're in a non-setup script context
    pub(crate) is_non_setup_script: bool,
    /// Compile demand skips browser-global and race reports. Full and doctor
    /// walks leave this false so those reports stay.
    pub(crate) skip_diagnostics: bool,
    /// Import statement spans in script content
    pub import_statements: Vec<ImportStatementInfo>,
    /// Re-export statement spans (`export { ... } from "..."`)
    pub re_exports: Vec<ReExportInfo>,
    /// Names re-exported from another module.
    pub re_export_forwards: Vec<crate::croquis::ReExportForward>,
    /// Components registered through Options API `components`.
    pub component_registrations: Vec<ComponentRegistration>,
    pub(crate) template_component_registrations: crate::croquis::TemplateComponentRegistrations,
    /// API shape of the component's default export (e.g. class component).
    pub component_shape: ComponentShape,
    /// Definition spans for bindings (name -> (start, end) offset in script)
    pub binding_spans: FxHashMap<CompactString, (u32, u32)>,
    /// Value import source by local binding name.
    pub import_sources: FxHashMap<CompactString, CompactString>,
    /// Normal `<script>` value exports that canon re-emits at module scope.
    /// Type exports may reference these through `typeof` without being
    /// demoted into `__setup()`.
    pub(crate) module_value_bindings: FxHashSet<CompactString>,
    /// Names referenced via `typeof X` in the body of each `type_exports`
    /// entry, indexed in parallel with `type_exports`. Used by
    /// `resolve_type_export_hoisting` to keep types adjacent to the
    /// setup-scope values they depend on. Pushed to in lockstep with
    /// `type_exports` via `record_type_export`.
    pub(crate) type_export_typeof_refs: Vec<FxHashSet<CompactString>>,
    /// Local type/interface identifiers referenced by each `type_exports`
    /// entry, indexed in parallel with `type_exports`. Used with
    /// `type_export_typeof_refs` to propagate setup-scope demotion through
    /// aliases such as `Props -> FormViewState -> typeof FormViewStateEnum`.
    pub(crate) type_export_type_refs: Vec<FxHashSet<CompactString>>,
    /// Static runtime object literal metadata available to macro spread args.
    pub(crate) runtime_object_literals: FxHashMap<CompactString, RuntimeObjectLiteral>,
    /// Authoritative Options API options-object descriptor, when resolved.
    pub(crate) options_descriptor: Option<OptionsDescriptor>,
    /// Whether component options disable attribute inheritance.
    pub inherit_attrs_disabled: bool,
    /// Reactive root of a member assignment while its right-hand side is walked.
    ///
    /// `open.value = [...open.value, index]` writes the copy back, so the spread
    /// is not a lost subscription.
    pub(crate) reactive_assignment_root: Option<CompactString>,
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
    /// Record a `TypeExport` together with type/value dependency references
    /// found in its body. Must be the only call site that pushes to
    /// `type_exports` so the parallel dependency vectors stay in lockstep for
    /// `resolve_type_export_hoisting`.
    pub(crate) fn record_type_export(&mut self, export: TypeExport, refs: TypeDependencyRefs) {
        if !refs.typeof_value_refs.is_empty() {
            self.refuse_occurrences();
        }
        self.type_exports.push(export);
        self.type_export_typeof_refs.push(refs.typeof_value_refs);
        self.type_export_type_refs.push(refs.type_refs);
    }

    /// Demote `TypeExport::hoisted` to `false` for any type whose body
    /// references a setup-scope value binding via `typeof`. The virtual TS
    /// generator only lifts hoisted types to module scope, so demoted types
    /// stay inside the synthetic `__setup` function alongside the values
    /// they depend on — which is the only place TS can resolve them.
    ///
    /// Imports and normal `<script>` named exports add value bindings at
    /// module scope, so `typeof moduleValue` references are left hoisted: those
    /// values are visible from the module scope where the type lands.
    pub(crate) fn resolve_type_export_hoisting(&mut self) {
        if self.type_exports.is_empty() {
            return;
        }

        let imported_names: FxHashSet<&str> = self
            .binding_spans
            .iter()
            .filter_map(|(name, (start, end))| {
                self.import_statements
                    .iter()
                    .any(|import| *start >= import.start && *end <= import.end)
                    .then_some(name.as_str())
            })
            .collect();

        for type_export in &mut self.type_exports {
            if imported_names.contains(type_export.name.as_str()) {
                type_export.hoisted = false;
            }
        }

        // A `typeof name` ref keeps a type hoisted only when `name` is visible
        // at module scope. Rather than materialize all imported binding names
        // up front (O(imports × bindings)), test each referenced name's
        // declaration span against the import ranges on demand — `refs` is
        // tiny, so this is O(refs × imports).
        for (idx, typeof_refs) in self.type_export_typeof_refs.iter().enumerate() {
            let touches_setup_value = typeof_refs.iter().any(|name| {
                let key = name.as_str();
                self.bindings.bindings.contains_key(key)
                    && !self.module_value_bindings.contains(key)
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

        let mut non_hoisted_type_names: FxHashSet<CompactString> = self
            .type_exports
            .iter()
            .filter(|te| !te.hoisted)
            .map(|te| te.name.clone())
            .collect();
        let mut changed = true;
        while changed {
            changed = false;
            for (type_export, type_refs) in self
                .type_exports
                .iter_mut()
                .zip(&self.type_export_type_refs)
            {
                if !type_export.hoisted {
                    continue;
                }

                let refs_non_hoisted_type = type_refs
                    .iter()
                    .any(|name| non_hoisted_type_names.contains(name));
                if refs_non_hoisted_type {
                    non_hoisted_type_names.insert(type_export.name.clone());
                    type_export.hoisted = false;
                    changed = true;
                }
            }
        }
    }

    /// Apply script analysis fields to an existing SFC analysis summary.
    ///
    /// This keeps script parsing as the single owner of script-scoped data while
    /// allowing callers to add template analysis before or after the script pass.
    pub fn apply_to_croquis(self, summary: &mut Croquis) {
        summary.bindings = self.bindings;
        summary.unused_bindings = self.unused_bindings;
        summary.macros = self.macros;
        summary.reactivity = self.reactivity;
        summary.race_conditions = self.race_conditions;
        summary.types = self.types;
        summary.type_exports = self.type_exports;
        summary.invalid_exports = self.invalid_exports;
        summary.scopes = self.scopes;
        summary.provide_inject = self.provide_inject;
        summary.setup_context = self.setup_context;
        summary.import_statements = self.import_statements;
        summary.re_exports = self.re_exports;
        summary.re_export_forwards = self.re_export_forwards;
        summary.component_registrations = self.component_registrations;
        summary.template_component_registrations = self.template_component_registrations;
        summary.component_shape = self.component_shape;
        summary.binding_spans = self.binding_spans;
        summary.options_descriptor = self.options_descriptor;
        summary.template_info.inherit_attrs_disabled |= self.inherit_attrs_disabled;
    }

    /// Convert script analysis into a `Croquis` summary.
    pub fn into_croquis(self) -> Croquis {
        let mut summary = Croquis::new();
        self.apply_to_croquis(&mut summary);
        summary
    }
}
