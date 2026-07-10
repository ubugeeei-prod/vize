//! Virtual TypeScript generation for standalone `.art.vue` files.

use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Drawer, DrawerOptions};

use super::super::{DiagnosticService, VirtualTsResult};
use super::virtual_ts::rewrite_vue_imports;
use super::virtual_ts_art_bindings::add_art_target_component_bindings;
use super::virtual_ts_art_imports::collect_art_vue_dependency_paths;

pub(in crate::ide::diagnostics) struct ArtVirtualTsResult {
    pub(in crate::ide::diagnostics) virtual_result: VirtualTsResult,
    pub(in crate::ide::diagnostics) vue_dependencies: Vec<PathBuf>,
}

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn generate_virtual_ts_for_art_with_dependencies(
        uri: &Url,
        content: &str,
    ) -> Option<ArtVirtualTsResult> {
        let art_allocator = vize_carton::Bump::new();
        let art_desc = vize_musea::parse_art(
            &art_allocator,
            content,
            vize_musea::ArtParseOptions::default(),
        )
        .ok()?;

        let (_, variant) = art_desc
            .variants
            .iter()
            .enumerate()
            .find(|(_, variant)| variant.is_default)
            .or_else(|| art_desc.variants.iter().enumerate().next())?;
        let template_content = variant.template;
        if template_content.trim().is_empty() {
            return None;
        }

        let template_ptr = template_content.as_ptr() as usize;
        let source_ptr = content.as_ptr() as usize;
        let template_offset = (template_ptr - source_ptr) as u32;

        let descriptor = vize_atelier_sfc::parse_sfc(
            content,
            vize_atelier_sfc::SfcParseOptions {
                filename: uri.path().to_string().into(),
                ..Default::default()
            },
        )
        .ok()?;

        let target_component = descriptor
            .script_setup
            .as_ref()
            .and_then(|script_setup| {
                crate::virtual_code::find_define_art_target_component(script_setup.content.as_ref())
            })
            .or_else(|| {
                art_desc
                    .metadata
                    .component
                    .and_then(crate::virtual_code::art_target_component_from_source)
            });

        let mut combined_script = String::new();
        let (script_offset, sfc_script_start_line) =
            if let Some(script_setup) = descriptor.script_setup.as_ref() {
                let isolate = !script_setup
                    .attrs
                    .get("isolate")
                    .is_some_and(|value| value.as_ref().eq_ignore_ascii_case("false"));
                let parts = crate::virtual_code::analyze_art_script_setup(
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    isolate,
                );

                for chunk in parts
                    .shared_imports
                    .iter()
                    .chain(parts.isolated_body.iter())
                {
                    combined_script.push_str(&chunk.text);
                    if !combined_script.ends_with('\n') {
                        combined_script.push('\n');
                    }
                }

                (
                    script_setup.loc.start as u32,
                    script_setup.loc.start_line as u32,
                )
            } else if let Some(script) = descriptor.script.as_ref() {
                combined_script.push_str(script.content.as_ref());
                if !combined_script.ends_with('\n') {
                    combined_script.push('\n');
                }
                (script.loc.start as u32, script.loc.start_line as u32)
            } else {
                (0, 1)
            };

        let script_content = combined_script.as_str();
        let template_allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&template_allocator, template_content);

        let mut analyzer = Drawer::with_options(DrawerOptions::full());
        analyzer.analyze_script(script_content);
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        let mut virtual_ts_options = VirtualTsOptions::default();
        if let Some(target) = target_component.as_ref() {
            add_art_target_component_bindings(&mut virtual_ts_options, &summary, target);
        }

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script_content),
            Some(&template_ast),
            script_offset,
            template_offset,
            &virtual_ts_options,
        );
        let code = output.code;
        let line_mappings = Self::parse_vize_map_comments(&code);
        let vue_dependencies = collect_art_vue_dependency_paths(uri, &code);
        let (rewritten_code, import_source_map) = rewrite_vue_imports(&code);

        Some(ArtVirtualTsResult {
            vue_dependencies,
            virtual_result: VirtualTsResult {
                code: rewritten_code,
                source_mappings: output.mappings,
                import_source_map,
                user_code_start_line: code
                    .lines()
                    .enumerate()
                    .find(|(_, line)| line.contains("// User setup code"))
                    .map(|(i, _)| i as u32 + 1)
                    .unwrap_or(0),
                sfc_script_start_line,
                template_scope_start_line: code
                    .lines()
                    .enumerate()
                    .find(|(_, line)| line.contains("Template Scope"))
                    .map(|(i, _)| i as u32)
                    .unwrap_or(u32::MAX),
                line_mappings,
                skipped_import_lines: Self::count_import_lines(script_content),
            },
        })
    }
}
