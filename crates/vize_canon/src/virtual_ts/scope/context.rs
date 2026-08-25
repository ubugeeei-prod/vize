//! Shared parameter-bundling contexts for recursive scope generation.

use vize_carton::CompactString;
use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;

use vize_croquis::{Croquis, EventHandlerScopeData, ScopeId, analysis::ComponentUsage};

use crate::virtual_ts::expressions::{ComponentPropSource, TemplateValueChecks};
use crate::virtual_ts::types::{VirtualTsCheckOptions, VirtualTsOptions};

use super::slot_outlet_props::SlotOutletChecks;

#[derive(Clone, Copy)]
pub(crate) enum GlobalComponentCheck {
    None,
    PascalCase,
    All,
}

impl GlobalComponentCheck {
    pub(crate) fn allows(self, name: &str) -> bool {
        match self {
            Self::None => false,
            Self::All => true,
            // Authored PascalCase tags are already safe TypeScript references.
            Self::PascalCase => name
                .as_bytes()
                .first()
                .is_some_and(|first| first.is_ascii_uppercase()),
        }
    }
}

/// Context for recursive scope generation, bundling shared parameters.
pub(crate) struct ScopeGenContext<'a, 'template> {
    pub(crate) summary: &'a Croquis,
    pub(crate) virtual_ts_options: &'a VirtualTsOptions,
    pub(crate) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(crate) skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) slot_outlets: &'a SlotOutletChecks,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(crate) checks: TemplateValueChecks<'a>,
    pub(crate) template_ast: Option<&'a vize_relief::RootNode<'template>>,
    pub(crate) template_source: Option<&'a str>,
    pub(crate) template_offset: u32,
    pub(crate) check_options: VirtualTsCheckOptions,
    pub(crate) legacy_vue2: bool,
}

pub(crate) struct ScopeGenerationOptions<'a, 'template> {
    pub(crate) check_options: VirtualTsCheckOptions,
    pub(crate) virtual_ts_options: &'a VirtualTsOptions,
    /// Names already declared by the conservative Options API setup-spread fallback.
    /// Instance-global generation shares the same template scope and must not redeclare them.
    pub(crate) setup_spread_bindings: &'a [String],
    pub(crate) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(crate) template_ast: Option<&'a vize_relief::RootNode<'template>>,
    pub(crate) check_unresolved_global_components: GlobalComponentCheck,
    pub(crate) legacy_vue2: bool,
    /// Options API generation declares `__default__`; template names outside
    /// the known bindings then resolve on the public instance (#3888).
    pub(crate) options_api: bool,
    pub(crate) preserve_event_navigation: bool,
    /// Whether the default-export rewrite declared the `__default__` alias. The
    /// public-instance form reads `typeof __default__`, so it stays off for a
    /// shape that never produced one (a re-exported default, no default export
    /// at all).
    pub(crate) has_default_alias: bool,
    /// Plain `<script>` content, when the component has one. Names it declares
    /// stay resolvable from template scope (a relocated `namespace`, a class, a
    /// plain binding), so those keep the free-name emission instead of the
    /// public-instance property access.
    pub(crate) script_content: Option<&'a str>,
}

/// Context for recursive component prop checks inside v-for scopes.
pub(crate) struct VForPropsContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) options: &'a VirtualTsOptions,
    pub(crate) components_by_scope: &'a FxHashMap<u32, Vec<(usize, &'a ComponentUsage)>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) vfor_enclosing_guards: &'a FxHashMap<u32, String>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(crate) source_context: ComponentPropSource<'a>,
    pub(crate) preserve_event_navigation: bool,
}

pub(super) struct EventHandlerExprContext<'a> {
    pub(super) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(super) data: &'a EventHandlerScopeData,
    /// Whether handler assignability and payload types should be checked.
    /// When disabled, authored handler bodies are still emitted as statements
    /// so template-binding checks remain valid and parseable.
    pub(super) check_emits: bool,
    pub(super) event_type: &'a str,
    pub(super) event_handler_type: Option<&'a str>,
    /// For component `@event` handlers, the synthesized listener type
    /// (e.g. `__Child_1_test_listener`) that captures the full emit argument
    /// tuple (or the single `$event` fallback when the emit stays unresolved).
    /// Native DOM events leave this `None` and fall back to the single `$event`
    /// parameter typed by `event_type`.
    pub(super) event_listener_type: Option<&'a str>,
    /// Authored range of the `@event` / `v-on:event` attribute name, where
    /// vue-tsc anchors a wrongly-shaped handler (#3462). `None` when the
    /// directive text cannot be read back from the template.
    pub(super) event_name_src_range: Option<std::ops::Range<usize>>,
    pub(super) template_prop_names: &'a FxHashSet<String>,
    pub(super) template_offset: u32,
    pub(super) indent: &'a str,
}

pub(super) struct ComponentPropsContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) template_source: Option<&'a str>,
    pub(super) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(super) vfor_enclosing_guards: &'a FxHashMap<u32, String>,
    pub(super) template_prop_names: &'a FxHashSet<String>,
    pub(super) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(super) template_offset: u32,
    pub(super) options: &'a VirtualTsOptions,
    pub(super) preserve_event_navigation: bool,
    pub(super) check_unresolved_global_components: GlobalComponentCheck,
    pub(super) legacy_vue2: bool,
    pub(super) check_unknown_props: bool,
}

impl<'a> ComponentPropsContext<'a> {
    pub(super) const fn source_context(&self) -> ComponentPropSource<'a> {
        ComponentPropSource::new(
            self.template_source,
            self.template_offset,
            &self.summary.scopes,
        )
    }
}
