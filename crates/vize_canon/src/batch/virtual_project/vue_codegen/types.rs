//! Input options and generated output shared by canonical Vue projections.

use super::super::setup_props::RuntimePropResolveCache;
use crate::{batch::Diagnostic, virtual_ts::VirtualTsCheckOptions};
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::{String as CompactString, config::VueVersion};

pub(in crate::batch::virtual_project) struct GeneratedVueFile {
    pub(in crate::batch::virtual_project) code: CompactString,
    pub(in crate::batch::virtual_project) mappings: Vec<crate::virtual_ts::VizeMapping>,
    pub(in crate::batch::virtual_project) semantic_links: Vec<crate::virtual_ts::VizeSemanticLink>,
    pub(in crate::batch::virtual_project) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
pub(in crate::batch::virtual_project) struct VueCodegenOptions<'a> {
    pub(in crate::batch::virtual_project) check_options: VirtualTsCheckOptions,
    pub(in crate::batch::virtual_project) preserve_unused_diagnostics: bool,
    pub(in crate::batch::virtual_project) options_api: bool,
    pub(in crate::batch::virtual_project) preserve_authored_component: bool,
    pub(in crate::batch::virtual_project) component_name: Option<&'a str>,
    pub(in crate::batch::virtual_project) preserve_event_navigation: bool,
    pub(in crate::batch::virtual_project) legacy_vue2: bool,
    pub(in crate::batch::virtual_project) dialect: VueVersion,
    pub(in crate::batch::virtual_project) template_syntax: TemplateSyntaxMode,
    pub(in crate::batch::virtual_project) experimental_in_tag_comments: bool,
    pub(in crate::batch::virtual_project) experimental_patterned_template: bool,
    pub(in crate::batch::virtual_project) experimental_strict_slot_children: bool,
    /// Hoist shared helpers to the batch ambient `.d.ts`; socket sessions keep
    /// them inline because they do not materialize that file.
    pub(in crate::batch::virtual_project) hoist_shared_preamble: bool,
    /// Content-mapper transforms can run outside Vite projects.
    pub(in crate::batch::virtual_project) omit_vite_client_reference: bool,
    /// Batch generation can share imported runtime prop/default resolution
    /// across rayon workers. Document/content-mapper callers use a per-file
    /// cache to avoid keeping stale source-derived state.
    pub(in crate::batch::virtual_project) runtime_prop_resolve_cache:
        Option<&'a RuntimePropResolveCache>,
}
