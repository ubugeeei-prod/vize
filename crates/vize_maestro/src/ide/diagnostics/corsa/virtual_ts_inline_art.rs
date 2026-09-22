//! Virtual TypeScript generation for inline `<art>` blocks in Vue SFCs.

use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::{
    VirtualTsOptions, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_legacy_vue2, generate_virtual_ts_with_offsets_options_api,
};

use super::super::{DiagnosticService, VirtualTsResult};

fn add_inline_self_component_binding(
    options: &mut VirtualTsOptions,
    summary: &vize_croquis::Croquis,
) {
    if summary
        .component_usages
        .iter()
        .any(|usage| usage.name.as_str() == "Self")
    {
        options
            .auto_import_stubs
            .push("declare const Self: { new (): { $props: Props } };".into());
        options.external_template_bindings.push("Self".into());
    }
}

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn generate_virtual_ts_for_inline_art_variants(
        uri: &Url,
        content: &str,
        options_api: bool,
        legacy_vue2: bool,
        base_options: &VirtualTsOptions,
    ) -> Vec<(usize, VirtualTsResult)> {
        if uri.path().ends_with(".art.vue") || !content.contains("<art") {
            return Vec::new();
        }

        let descriptor = match vize_atelier_sfc::parse_sfc(
            content,
            vize_atelier_sfc::SfcParseOptions {
                filename: uri.path().to_string().into(),
                ..Default::default()
            },
        ) {
            Ok(descriptor) => descriptor,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        let mut variant_index = 0usize;

        for custom in &descriptor.custom_blocks {
            if custom.block_type != "art" {
                continue;
            }

            let variants =
                crate::virtual_code::inline_art_variants(custom.content.as_ref(), custom.loc.start);
            if variants.is_empty() {
                continue;
            }

            for variant in variants {
                let current_variant_index = variant_index;
                variant_index += 1;

                let Some(template_content) =
                    content.get(variant.template_start..variant.template_end)
                else {
                    continue;
                };
                if template_content.trim().is_empty() {
                    continue;
                }

                let template_allocator = vize_s0::Allocator::new();
                let (template_ast, _) = vize_armature::parse(&template_allocator, template_content);
                let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor_resolved(
                    &descriptor,
                    Some(&template_ast),
                    vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
                    options_api,
                    legacy_vue2,
                    uri.path(),
                );

                let script_content = analysis.script_content.unwrap_or_default();
                let script_offset = analysis.script_offset;
                let mut virtual_ts_options = base_options.clone();
                add_inline_self_component_binding(&mut virtual_ts_options, &analysis.croquis);

                let generate_virtual_ts = if legacy_vue2 {
                    generate_virtual_ts_with_offsets_legacy_vue2
                } else if options_api {
                    generate_virtual_ts_with_offsets_options_api
                } else {
                    generate_virtual_ts_with_offsets
                };
                let output = generate_virtual_ts(
                    &analysis.croquis,
                    Some(script_content.as_str()),
                    Some(&template_ast),
                    script_offset,
                    variant.template_start as u32,
                    &virtual_ts_options,
                );
                let code = output.code;
                let (source_mappings, semantic_links) = output.mapping.into_parts();

                results.push((
                    current_variant_index,
                    VirtualTsResult {
                        code: code.to_string(),
                        source_mappings,
                        semantic_links,
                        import_source_map: Default::default(),
                    },
                ));
            }
        }

        results
    }
}
