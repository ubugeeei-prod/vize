//! Shared parameter-bundling contexts for recursive scope generation.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;

use vize_croquis::{Croquis, EventHandlerScopeData, ScopeId, analysis::ComponentUsage};

use crate::virtual_ts::expressions::{ComponentPropSource, TemplateValueChecks};
use crate::virtual_ts::types::{VirtualTsCheckOptions, VirtualTsOptions};

/// Context for recursive scope generation, bundling shared parameters.
pub(crate) struct ScopeGenContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) virtual_ts_options: &'a VirtualTsOptions,
    pub(crate) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(crate) skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) checks: TemplateValueChecks<'a>,
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
    pub(crate) template_ast: Option<&'a vize_relief::RootNode<'template>>,
    pub(crate) check_unresolved_global_components: bool,
    pub(crate) legacy_vue2: bool,
}

/// Context for recursive component prop checks inside v-for scopes.
pub(crate) struct VForPropsContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) options: &'a VirtualTsOptions,
    pub(crate) components_by_scope: &'a FxHashMap<u32, Vec<(usize, &'a ComponentUsage)>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) vfor_enclosing_guards: &'a FxHashMap<u32, String>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) source_context: ComponentPropSource<'a>,
}

pub(super) struct EventHandlerExprContext<'a> {
    pub(super) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(super) data: &'a EventHandlerScopeData,
    /// Whether handler assignability and payload types should be checked.
    /// When disabled, authored handler bodies are still emitted as statements
    /// so template-binding checks remain valid and parseable.
    pub(super) check_emits: bool,
    pub(super) event_type: &'a str,
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
    pub(super) template_offset: u32,
    pub(super) options: &'a VirtualTsOptions,
    pub(super) check_unresolved_global_components: bool,
    pub(super) legacy_vue2: bool,
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
