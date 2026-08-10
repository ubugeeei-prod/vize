//! Public entry points for virtual TypeScript generation: thin wrappers that
//! fix offsets and generation flags before delegating to
//! [`super::generate_virtual_ts_with_offsets_and_checks`].

use vize_croquis::Croquis;

use crate::virtual_ts::types::{VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput};

use super::generate_virtual_ts_with_offsets_and_checks;

/// Generate virtual TypeScript from Vue SFC analysis.
///
/// This ensures compiler macros like defineProps are ONLY valid in setup scope.
pub fn generate_virtual_ts(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    template_offset: u32,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets(
        summary,
        script_content,
        template_ast,
        0,
        template_offset,
        &VirtualTsOptions::default(),
    )
}

/// Generate virtual TypeScript with explicit script and template offsets.
///
/// `script_offset` is the byte offset of the script content within the SFC file.
/// `template_offset` is the byte offset of the template content within the SFC file.
/// When these are provided, source mappings point to SFC-absolute positions.
/// `options` controls template globals and other generation settings.
pub fn generate_virtual_ts_with_offsets(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions::default(),
    )
}

/// Generate virtual TypeScript with Vue 3 Options API binding resolution
/// enabled (opt-in, standard build — no `legacy` feature required).
pub fn generate_virtual_ts_with_offsets_options_api(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions {
            options_api: true,
            preserve_authored_component: true,
            ..Default::default()
        },
    )
}
