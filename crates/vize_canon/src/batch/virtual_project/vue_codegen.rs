//! Shared output types for Atlas-backed Vue virtual TypeScript emission.

use vize_carton::{String as CompactString, config::VueVersion};
use vize_relief::TemplateSyntaxMode;

use crate::batch::Diagnostic;
use crate::virtual_ts::{VirtualTsCheckOptions, VizeMapping};

pub(super) struct GeneratedVueFile {
    pub(super) code: CompactString,
    pub(super) mappings: Vec<VizeMapping>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy)]
pub(super) struct VueCodegenOptions {
    pub(super) check_options: VirtualTsCheckOptions,
    pub(super) preserve_unused_diagnostics: bool,
    pub(super) options_api: bool,
    pub(super) legacy_vue2: bool,
    pub(super) dialect: VueVersion,
    pub(super) template_syntax: TemplateSyntaxMode,
    /// Hoist shared helpers to the batch ambient `.d.ts`; socket sessions keep
    /// them inline because they do not materialize that file.
    pub(super) hoist_shared_preamble: bool,
}
