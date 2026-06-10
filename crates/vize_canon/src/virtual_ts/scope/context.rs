//! Shared parameter-bundling contexts for recursive scope generation.

use vize_carton::FxHashMap;
use vize_carton::FxHashSet;
use vize_carton::String;

use vize_croquis::{Croquis, EventHandlerScopeData, ScopeId, analysis::ComponentUsage};

use crate::virtual_ts::types::{VirtualTsCheckOptions, VirtualTsOptions};

/// Context for recursive scope generation, bundling shared parameters.
pub(crate) struct ScopeGenContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) template_offset: u32,
    pub(crate) check_options: VirtualTsCheckOptions,
    pub(crate) template_syntax_quirks: bool,
}

pub(crate) struct ScopeGenerationOptions<'a> {
    pub(crate) check_options: VirtualTsCheckOptions,
    pub(crate) virtual_ts_options: &'a VirtualTsOptions,
    pub(crate) check_unresolved_global_components: bool,
    pub(crate) template_syntax_quirks: bool,
}

/// Context for recursive component prop checks inside v-for scopes.
pub(crate) struct VForPropsContext<'a> {
    pub(crate) summary: &'a Croquis,
    pub(crate) components_by_scope: &'a FxHashMap<u32, Vec<(usize, &'a ComponentUsage)>>,
    pub(crate) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(crate) template_prop_names: &'a FxHashSet<String>,
    pub(crate) template_offset: u32,
}

pub(super) struct EventHandlerExprContext<'a> {
    pub(super) expressions_by_scope: &'a FxHashMap<u32, Vec<&'a vize_croquis::TemplateExpression>>,
    pub(super) data: &'a EventHandlerScopeData,
    pub(super) event_type: &'a str,
    pub(super) template_prop_names: &'a FxHashSet<String>,
    pub(super) template_offset: u32,
    pub(super) indent: &'a str,
}

pub(super) struct ComponentPropsContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) children_map: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub(super) template_prop_names: &'a FxHashSet<String>,
    pub(super) template_offset: u32,
    pub(super) options: &'a VirtualTsOptions,
    pub(super) check_unresolved_global_components: bool,
}
