//! Virtual TypeScript generation from Vue SFCs and `.art.vue` files.

use tower_lsp::lsp_types::Url;

use super::super::{DiagnosticService, SourceMapping, VirtualTsResult};
use vize_canon::{ImportRewriter, ImportSourceMap};

/// Apply `ImportRewriter` to the generated virtual TS so `.vue` imports
/// resolve to the generated `.vue.ts` mirrors in the editor Corsa session.
///
/// The rewrite only changes bytes *inside* import specifier strings (single
/// line), so line numbers are preserved — only column offsets within affected
/// lines shift. Returns the rewritten code and a byte-offset source map that
/// `map_diagnostic_with_source_mappings` uses to translate post-rewrite
/// diagnostic offsets back into pre-rewrite virtual TS offsets (which are the
/// coordinate system the byte-range source mappings operate in).
fn rewrite_vue_imports(code: &str) -> (std::string::String, ImportSourceMap) {
    use oxc_span::SourceType;
    let result = ImportRewriter::new().rewrite(code, SourceType::ts());
    #[allow(clippy::disallowed_methods)]
    (result.code.to_string(), result.source_map)
}

fn collect_relative_vue_specifiers(code: &str) -> Vec<std::string::String> {
    use oxc_span::SourceType;
    #[allow(clippy::disallowed_methods)]
    ImportRewriter::new()
        .collect_relative_vue_specifiers(code, SourceType::ts())
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

impl DiagnosticService {
    /// Generate virtual TypeScript for a Vue SFC.
    pub(in crate::ide::diagnostics) fn generate_virtual_ts(
        uri: &Url,
        content: &str,
        options_api: bool,
        legacy_vue2: bool,
    ) -> Option<VirtualTsResult> {
        use vize_atelier_sfc::{SfcParseOptions, croquis::SfcCroquisOptions, parse_sfc};
        use vize_canon::virtual_ts::{
            VirtualTsOptions, generate_virtual_ts_with_offsets,
            generate_virtual_ts_with_offsets_legacy_vue2,
            generate_virtual_ts_with_offsets_options_api,
        };

        let options = SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = parse_sfc(content, options).ok()?;

        let template_block = descriptor.template.as_ref()?;
        let template_offset = template_block.loc.start as u32;

        let allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&allocator, &template_block.content);

        let croquis_options = SfcCroquisOptions::full();
        // Croquis cannot resolve props inherited through imported/heritage
        // types; the resolved analysis merges the script compile context's
        // props before the template pass so the virtual TS fed to Corsa sees
        // the full prop set.
        let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor_resolved(
            &descriptor,
            Some(&template_ast),
            croquis_options,
            options_api,
            legacy_vue2,
            uri.path(),
        );

        let script_content = analysis.script_content?;
        let script_offset = analysis.script_offset;
        let sfc_script_start_line =
            crate::ide::offset_to_position(content, script_offset as usize).0 + 1;

        let virtual_ts_options = VirtualTsOptions::default();
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
            template_offset,
            &virtual_ts_options,
        );
        let code = output.code;
        let source_mappings = output.mappings;

        // Count import lines in script content (these are moved to module scope)
        // Import lines are skipped from user setup code section
        let skipped_import_lines = Self::count_import_lines(script_content.as_str());

        // Find where user code starts in generated virtual TS
        // Look for "// User setup code" comment
        let user_code_start_line = code
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("// User setup code"))
            .map(|(i, _)| i as u32 + 1) // +1 because user code is on next line
            .unwrap_or(0);

        // Find where template scope starts in generated virtual TS
        // Look for "// Template Scope" or "// ========== Template Scope" comment
        let template_scope_start_line = code
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("Template Scope"))
            .map(|(i, _)| i as u32)
            .unwrap_or(u32::MAX);

        // Parse @vize-map comments to build line mappings
        // Format: // @vize-map: TYPE -> START:END
        // Where START:END are byte offsets in the SFC
        // @vize-map comments are inserted by the generator on dedicated lines,
        // so line indices survive the `.vue` → `.vue.ts` import rewrite below
        // (which only edits bytes inside string literals on import lines).
        let line_mappings = Self::parse_vize_map_comments(&code);

        // Issue #752: rewrite `.vue` import specifiers to `.vue.ts` so the
        // editor's Corsa session resolves sibling SFCs via the same virtual
        // mirrors used by the batch path. Collect the relative specifiers
        // from the pre-rewrite code so the caller can overlay siblings.
        let relative_vue_imports = collect_relative_vue_specifiers(&code);
        let (rewritten_code, import_source_map) = rewrite_vue_imports(&code);

        Some(VirtualTsResult {
            code: rewritten_code,
            source_mappings,
            import_source_map,
            relative_vue_imports,
            user_code_start_line,
            sfc_script_start_line,
            template_scope_start_line,
            line_mappings,
            skipped_import_lines,
        })
    }

    /// Count the number of import lines in script content.
    /// Handles multi-line imports.
    pub(in crate::ide::diagnostics) fn count_import_lines(script: &str) -> u32 {
        let lines: Vec<&str> = script.lines().collect();
        let mut count = 0u32;
        let mut in_import = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("import ") {
                in_import = true;
                count += 1;
                // Check if this is a single-line import
                if trimmed.ends_with(';') || trimmed.contains(" from ") {
                    in_import = false;
                }
            } else if in_import {
                count += 1;
                // Check if this line ends the import
                if trimmed.ends_with(';') {
                    in_import = false;
                }
            }
        }

        count
    }

    /// Parse @vize-map comments from generated virtual TS code.
    /// Returns a vector where index is line number and value is source mapping.
    pub(in crate::ide::diagnostics) fn parse_vize_map_comments(
        code: &str,
    ) -> Vec<Option<SourceMapping>> {
        let mut mappings: Vec<Option<SourceMapping>> = vec![None; code.lines().count()];
        let mut found_count = 0;

        // Parse @vize-map comments without regex
        // Format: // @vize-map: TYPE -> START:END
        for (line_idx, line) in code.lines().enumerate() {
            // Find @vize-map comment
            if let Some(map_idx) = line.find("@vize-map:") {
                // Extract the part after @vize-map:
                let rest = &line[map_idx + "@vize-map:".len()..];

                // Find -> separator
                if let Some(arrow_idx) = rest.find("->") {
                    // Extract START:END part after ->
                    let offsets_part = rest[arrow_idx + 2..].trim();

                    // Parse START:END
                    if let Some(colon_idx) = offsets_part.find(':') {
                        let start_str = offsets_part[..colon_idx].trim();
                        let end_str = offsets_part[colon_idx + 1..].trim();

                        // Remove any trailing non-digit characters
                        let end_str = end_str
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect::<String>();

                        if let (Ok(start_val), Ok(end_val)) =
                            (start_str.parse::<u32>(), end_str.parse::<u32>())
                        {
                            // The mapping applies to the line BEFORE the comment
                            // (the actual code that will produce the error)
                            if line_idx > 0 {
                                mappings[line_idx - 1] = Some(SourceMapping {
                                    start: start_val,
                                    end: end_val,
                                });
                                found_count += 1;
                                tracing::debug!(
                                    "vize-map: line {} -> offset {}:{} (from: {})",
                                    line_idx - 1,
                                    start_val,
                                    end_val,
                                    &line[..line.len().min(80)]
                                );
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("parse_vize_map_comments: found {} mappings", found_count);
        mappings
    }

    /// Generate virtual TypeScript for an art file (*.art.vue).
    ///
    /// Uses the default variant's template as the synthetic template,
    /// and the script_setup block from the SFC parse.
    pub(in crate::ide::diagnostics) fn generate_virtual_ts_for_art(
        uri: &Url,
        content: &str,
    ) -> Option<VirtualTsResult> {
        use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
        use vize_canon::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
        use vize_croquis::{Drawer, DrawerOptions};

        // Parse as art file to get variant templates
        let art_allocator = vize_carton::Bump::new();
        let art_desc = vize_musea::parse_art(
            &art_allocator,
            content,
            vize_musea::ArtParseOptions::default(),
        )
        .ok()?;

        // Get default variant's template
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

        // Calculate template offset in the original art file
        let template_ptr = template_content.as_ptr() as usize;
        let source_ptr = content.as_ptr() as usize;
        let template_offset = (template_ptr - source_ptr) as u32;

        // Parse SFC for script blocks
        let sfc_options = SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };
        let descriptor = parse_sfc(content, sfc_options).ok()?;

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
                return None;
            };

        let script_content = combined_script.as_str();

        // Parse template AST
        let template_allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&template_allocator, template_content);

        // Analyze script + template
        let mut analyzer = Drawer::with_options(DrawerOptions::full());
        analyzer.analyze_script(script_content);
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script_content),
            Some(&template_ast),
            script_offset,
            template_offset,
            &VirtualTsOptions::default(),
        );
        let code = output.code;
        let source_mappings = output.mappings;

        // Count import lines
        let skipped_import_lines = Self::count_import_lines(script_content);

        // Find where user code starts
        let user_code_start_line = code
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("// User setup code"))
            .map(|(i, _)| i as u32 + 1)
            .unwrap_or(0);

        // Find where template scope starts
        let template_scope_start_line = code
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("Template Scope"))
            .map(|(i, _)| i as u32)
            .unwrap_or(u32::MAX);

        // Parse @vize-map comments
        let line_mappings = Self::parse_vize_map_comments(&code);

        // Issue #752: same rewrite as the non-art path so `.vue` imports in
        // the art file's `<script setup>` resolve to virtual `.vue.ts` mirrors.
        let relative_vue_imports = collect_relative_vue_specifiers(&code);
        let (rewritten_code, import_source_map) = rewrite_vue_imports(&code);

        Some(VirtualTsResult {
            code: rewritten_code,
            source_mappings,
            import_source_map,
            relative_vue_imports,
            user_code_start_line,
            sfc_script_start_line,
            template_scope_start_line,
            line_mappings,
            skipped_import_lines,
        })
    }
}
