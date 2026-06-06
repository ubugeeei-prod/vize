//! Corsa integration for collecting native TypeScript diagnostics.
//!
//! This module generates virtual TypeScript from Vue SFCs and uses the Corsa
//! LSP bridge to collect type-checking diagnostics.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use crate::server::ServerState;

use super::{DiagnosticService, SourceMapping, VirtualTsResult, sources};
use vize_canon::{ImportRewriter, ImportSourceMap};
use vize_carton::cstr;

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

/// Overlay the virtual TS for every relative `.vue` import of `host_uri`
/// into the editor's Corsa session so TypeScript module resolution can find
/// them (issue #752). Each sibling is opened at `<sibling_abs_path>.ts`,
/// matching the `.vue.ts` suffix produced by `ImportRewriter`. Transitive
/// `.vue` imports are followed recursively; cycles are avoided via a
/// visited set keyed on the canonicalized sibling path. Failures are
/// logged and skipped — a missing sibling falls through to the existing
/// TS2307 surface, which is the desired behavior for genuinely missing
/// modules.
async fn overlay_sibling_vue_mirrors(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    host_uri: &Url,
    initial_specifiers: &[std::string::String],
    legacy_vue2: bool,
) {
    use std::collections::HashSet;
    use std::path::PathBuf;

    if initial_specifiers.is_empty() {
        return;
    }

    let Ok(host_path) = host_uri.to_file_path() else {
        tracing::debug!("overlay_sibling_vue_mirrors: host URI is not a file path: {host_uri}",);
        return;
    };
    let host_dir = match host_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => return,
    };

    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(host_path.clone());

    let mut queue: Vec<(PathBuf, Vec<std::string::String>)> =
        vec![(host_dir, initial_specifiers.to_vec())];

    while let Some((dir, specifiers)) = queue.pop() {
        for specifier in specifiers {
            let resolved = dir.join(specifier.as_str());
            // Canonicalize is best-effort: if the file doesn't exist we still
            // try the lexical join so genuinely missing imports surface
            // TS2307 just as before.
            let canonical = std::fs::canonicalize(&resolved).unwrap_or(resolved);
            if !visited.insert(canonical.clone()) {
                continue;
            }

            let sibling_content = match std::fs::read_to_string(&canonical) {
                Ok(text) => text,
                Err(err) => {
                    tracing::debug!(
                        "overlay sibling skipped — read failed for {}: {err}",
                        canonical.display(),
                    );
                    continue;
                }
            };

            let sibling_uri = match Url::from_file_path(&canonical) {
                Ok(uri) => uri,
                Err(_) => continue,
            };

            let sibling_virtual = if canonical.to_string_lossy().ends_with(".art.vue") {
                DiagnosticService::generate_virtual_ts_for_art(&sibling_uri, &sibling_content)
            } else {
                DiagnosticService::generate_virtual_ts(&sibling_uri, &sibling_content, legacy_vue2)
            };
            let Some(sibling_virtual) = sibling_virtual else {
                continue;
            };

            let sibling_name = cstr!("{}.ts", canonical.to_string_lossy());
            if let Err(err) = bridge
                .open_or_update_virtual_document(&sibling_name, &sibling_virtual.code)
                .await
            {
                tracing::debug!("overlay sibling failed for {}: {err}", canonical.display(),);
                continue;
            }

            let next_dir = canonical
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| canonical.clone());
            if !sibling_virtual.relative_vue_imports.is_empty() {
                queue.push((next_dir, sibling_virtual.relative_vue_imports));
            }
        }
    }
}

type LspRangeParts = (u32, u32, u32, u32);

impl DiagnosticService {
    /// Collect diagnostics from the Corsa project-session backend.
    pub(super) async fn collect_corsa_diagnostics(
        state: &ServerState,
        uri: &Url,
    ) -> Vec<Diagnostic> {
        tracing::info!("collect_corsa_diagnostics: {}", uri);

        // Only process .vue files
        if !uri.path().ends_with(".vue") {
            tracing::debug!("skipping non-vue file: {}", uri);
            return vec![];
        }

        // Get document content
        let Some(doc) = state.documents.get(uri) else {
            tracing::warn!("document not found: {}", uri);
            return vec![];
        };
        let content = doc.text();

        // Get the shared Corsa bridge.
        tracing::info!("getting corsa bridge...");
        let Some(bridge) = state.get_corsa_bridge().await else {
            tracing::warn!("corsa bridge not available");
            return vec![];
        };
        tracing::info!("corsa bridge acquired");

        // Generate virtual TypeScript
        let is_art_file = uri.path().ends_with(".art.vue");
        let legacy_vue2 = state.legacy_vue2_enabled();
        let virtual_result = if is_art_file {
            Self::generate_virtual_ts_for_art(uri, &content)
        } else {
            Self::generate_virtual_ts(uri, &content, legacy_vue2)
        };
        let Some(virtual_result) = virtual_result else {
            tracing::warn!("failed to generate virtual ts for {}", uri);
            return vec![];
        };
        let virtual_ts = &virtual_result.code;
        let user_code_start_line = virtual_result.user_code_start_line;
        let sfc_script_start_line = virtual_result.sfc_script_start_line;
        let template_scope_start_line = virtual_result.template_scope_start_line;
        let line_mappings = &virtual_result.line_mappings;
        let source_mappings = &virtual_result.source_mappings;
        tracing::info!(
            "generated virtual ts ({} bytes), user_code_start={}, sfc_script_start={}, template_scope_start={}, mappings_count={}",
            virtual_ts.len(),
            user_code_start_line,
            sfc_script_start_line,
            template_scope_start_line,
            line_mappings.iter().filter(|m| m.is_some()).count()
        );

        // Create the virtual document name used to derive a stable URI.
        let virtual_name = cstr!("{}.ts", uri.path());

        // Issue #752: Overlay sibling `.vue.ts` mirrors for every relative
        // `.vue` import so TypeScript's module resolution succeeds against
        // the temp-dir Corsa session. Without this, `import App from
        // './app.vue'` rewrites correctly to `./app.vue.ts` but still
        // reports TS2307 because the sibling is not present.
        overlay_sibling_vue_mirrors(
            &bridge,
            uri,
            &virtual_result.relative_vue_imports,
            legacy_vue2,
        )
        .await;

        // Open or update the document in Corsa (uses didChange if already open).
        tracing::info!("opening/updating virtual document: {}", virtual_name);
        let virtual_uri = match bridge
            .open_or_update_virtual_document(&virtual_name, virtual_ts)
            .await
        {
            Ok(uri) => {
                tracing::info!("virtual document opened/updated successfully: {}", uri);
                uri
            }
            Err(e) => {
                tracing::warn!("failed to open/update virtual document: {}", e);
                return vec![];
            }
        };

        // Get diagnostics (this polls until publishDiagnostics has landed).
        tracing::info!(
            "waiting for diagnostics from corsa bridge for {}",
            virtual_uri
        );
        let Ok(corsa_diags) = bridge.get_diagnostics(&virtual_uri).await else {
            tracing::warn!("failed to get diagnostics from corsa");
            return vec![];
        };

        tracing::info!(
            "corsa returned {} raw diagnostics for {}",
            corsa_diags.len(),
            virtual_uri
        );

        // Log each diagnostic for debugging
        for (i, diag) in corsa_diags.iter().enumerate() {
            tracing::info!(
                "  raw diag[{}]: line {}-{}, message: {}",
                i,
                diag.range.start.line,
                diag.range.end.line,
                &diag.message[..diag.message.len().min(100)]
            );
        }

        // Convert to LSP diagnostics with proper position mapping
        corsa_diags
            .into_iter()
            .filter_map(|diag| {
                // Skip warnings about internal generated variables
                // TS6133: 'X' is declared but its value is never read
                // TS6196: 'X' is declared but never used
                let is_unused_warning = diag.message.contains("is declared but")
                    && (diag.message.contains("never read") || diag.message.contains("never used"));
                let is_internal_var = diag.message.contains("'__")
                    || diag.message.contains("'$event'")
                    || diag.message.contains("'$attrs'")
                    || diag.message.contains("'$slots'")
                    || diag.message.contains("'$refs'")
                    || diag.message.contains("'$emit'");

                if is_unused_warning && is_internal_var {
                    tracing::debug!(
                        "skipping internal variable warning: {}",
                        &diag.message[..diag.message.len().min(80)]
                    );
                    return None;
                }

                let mapped_range = map_diagnostic_with_source_mappings(
                    virtual_ts,
                    content.as_str(),
                    source_mappings,
                    &virtual_result.import_source_map,
                    diag.range.start.line,
                    diag.range.start.character,
                    diag.range.end.line,
                    diag.range.end.character,
                );

                // Determine if this is a script error or template error.
                // Prefer byte-range source maps from canon. The older line
                // mapping remains as a fallback for diagnostics that land on
                // synthetic wrapper statements.
                let is_template_error = diag.range.start.line >= template_scope_start_line;

                let (start_line, end_line, start_char, end_char) = if let Some(mapped_range) = mapped_range {
                    mapped_range
                } else if is_template_error {
                    // Template scope error - try to find source mapping from @vize-map comments
                    let virtual_line = diag.range.start.line as usize;

                    // @vize-map comments are placed AFTER the code line they map.
                    // So for an error at line N, the mapping is at line N (from comment at N+1).
                    // Search forward (down) from the error line to find the mapping.
                    let mapping = (0..=10)
                        .filter_map(|offset| {
                            let search_line = virtual_line + offset;
                            line_mappings.get(search_line).and_then(|m| m.as_ref())
                        })
                        .next();

                    if let Some(src_mapping) = mapping {
                        // Found a source mapping - convert byte offset to line/column
                        let (start_line, start_col) =
                            source_offset_to_position(&content, src_mapping.start as usize);
                        let (end_line, end_col) =
                            source_offset_to_position(&content, src_mapping.end as usize);

                        tracing::info!(
                            "template error with mapping: virtual_line={} -> offset {}:{} -> sfc_line={} (message: {})",
                            diag.range.start.line,
                            src_mapping.start,
                            src_mapping.end,
                            start_line,
                            &diag.message[..diag.message.len().min(50)]
                        );
                        (start_line, end_line, start_col, end_col)
                    } else {
                        // No mapping found - skip this diagnostic
                        tracing::debug!(
                            "skipping unmapped template error at line {}: {}",
                            diag.range.start.line,
                            &diag.message[..diag.message.len().min(50)]
                        );
                        return None;
                    }
                } else {
                    // Skip diagnostics in generated preamble when no source map
                    // points back to user code.
                    if diag.range.start.line < user_code_start_line {
                        tracing::debug!(
                            "skipping preamble diagnostic at line {} (user code starts at {}): {}",
                            diag.range.start.line,
                            user_code_start_line,
                            &diag.message[..diag.message.len().min(50)]
                        );
                        return None;
                    }

                    // Script error - map using user code offset
                    let user_code_offset =
                        diag.range.start.line.saturating_sub(user_code_start_line);
                    let user_code_offset_end =
                        diag.range.end.line.saturating_sub(user_code_start_line);

                    // sfc_script_start_line is 1-indexed, convert to 0-indexed
                    // Add skipped_import_lines to account for import lines that were moved to module scope
                    let skipped_lines = virtual_result.skipped_import_lines;
                    let start =
                        (sfc_script_start_line.saturating_sub(1)) + user_code_offset + skipped_lines;
                    let end = (sfc_script_start_line.saturating_sub(1))
                        + user_code_offset_end
                        + skipped_lines;

                    // Adjust character offset: virtual TS adds 2 spaces of indentation
                    let start_ch = diag.range.start.character.saturating_sub(2);
                    let end_ch = diag.range.end.character.saturating_sub(2);

                    tracing::debug!(
                        "script error: virtual_line={} -> sfc_line={} (skipped_imports={}, message: {})",
                        diag.range.start.line,
                        start,
                        skipped_lines,
                        &diag.message[..diag.message.len().min(50)]
                    );
                    (start, end, start_ch, end_ch)
                };

                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_char,
                        },
                        end: Position {
                            line: end_line,
                            character: end_char,
                        },
                    },
                    severity: diag.severity.map(|s| match s {
                        1 => DiagnosticSeverity::ERROR,
                        2 => DiagnosticSeverity::WARNING,
                        3 => DiagnosticSeverity::INFORMATION,
                        _ => DiagnosticSeverity::HINT,
                    }),
                    source: Some(sources::TYPE_CHECKER.to_string()),
                    message: rewrite_corsa_message(&diag.message),
                    ..Default::default()
                })
            })
            .collect()
    }

    /// Generate virtual TypeScript for a Vue SFC.
    pub(super) fn generate_virtual_ts(
        uri: &Url,
        content: &str,
        legacy_vue2: bool,
    ) -> Option<VirtualTsResult> {
        use vize_atelier_sfc::{
            SfcParseOptions,
            croquis::{
                SfcCroquisOptions, analyze_sfc_descriptor_with_context,
                analyze_sfc_descriptor_with_context_legacy_vue2,
            },
            parse_sfc,
        };
        use vize_canon::virtual_ts::{
            VirtualTsOptions, generate_virtual_ts_with_offsets,
            generate_virtual_ts_with_offsets_legacy_vue2,
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
        let analysis = if legacy_vue2 {
            analyze_sfc_descriptor_with_context_legacy_vue2(
                &descriptor,
                Some(&template_ast),
                croquis_options,
            )
        } else {
            analyze_sfc_descriptor_with_context(&descriptor, Some(&template_ast), croquis_options)
        };
        let script_content = analysis.script_content?;
        let script_offset = analysis.script_offset;
        let sfc_script_start_line =
            crate::ide::offset_to_position(content, script_offset as usize).0 + 1;

        let virtual_ts_options = VirtualTsOptions::default();
        let generate_virtual_ts = if legacy_vue2 {
            generate_virtual_ts_with_offsets_legacy_vue2
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
    pub(super) fn count_import_lines(script: &str) -> u32 {
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
    pub(super) fn parse_vize_map_comments(code: &str) -> Vec<Option<SourceMapping>> {
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
    pub(super) fn generate_virtual_ts_for_art(uri: &Url, content: &str) -> Option<VirtualTsResult> {
        use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
        use vize_canon::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
        use vize_croquis::{Analyzer, AnalyzerOptions};

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
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
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

#[allow(clippy::too_many_arguments)]
fn map_diagnostic_with_source_mappings(
    virtual_ts: &str,
    source: &str,
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    import_source_map: &ImportSourceMap,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> Option<LspRangeParts> {
    // Diagnostics come back from Corsa in coordinates of the *rewritten*
    // virtual TS (the one we sent). The byte-range mappings, however, were
    // produced before the `.vue` → `.vue.ts` rewrite. Translate first.
    let start_offset_post = line_character_to_byte_offset(virtual_ts, start_line, start_character)?;
    let end_offset_post = line_character_to_byte_offset(virtual_ts, end_line, end_character)
        .unwrap_or(start_offset_post.saturating_add(1));
    let start_offset = import_source_map.get_original_offset(start_offset_post as u32) as usize;
    let end_offset = import_source_map.get_original_offset(end_offset_post as u32) as usize;
    let start_mapping = mapping_for_generated_offset(mappings, start_offset)?;
    let src_start = map_generated_offset_to_source(start_mapping, start_offset);
    let src_end = mapping_for_generated_offset(mappings, end_offset)
        .map(|mapping| map_generated_offset_to_source(mapping, end_offset))
        .unwrap_or_else(|| {
            let generated_len = end_offset.saturating_sub(start_offset);
            src_start
                .saturating_add(generated_len)
                .min(start_mapping.src_range.end)
        })
        .max(src_start.saturating_add(1));

    let (start_line, start_char) = source_offset_to_position(source, src_start);
    let (end_line, end_char) = source_offset_to_position(source, src_end.min(source.len()));
    Some((start_line, end_line, start_char, end_char))
}

fn mapping_for_generated_offset(
    mappings: &[vize_canon::virtual_ts::VizeMapping],
    offset: usize,
) -> Option<&vize_canon::virtual_ts::VizeMapping> {
    mappings
        .iter()
        .find(|mapping| offset >= mapping.gen_range.start && offset <= mapping.gen_range.end)
}

fn map_generated_offset_to_source(
    mapping: &vize_canon::virtual_ts::VizeMapping,
    generated_offset: usize,
) -> usize {
    let generated_relative = generated_offset.saturating_sub(mapping.gen_range.start);
    let source_len = mapping
        .src_range
        .end
        .saturating_sub(mapping.src_range.start);
    mapping
        .src_range
        .start
        .saturating_add(generated_relative.min(source_len.saturating_sub(1)))
}

fn line_character_to_byte_offset(text: &str, line: u32, character: u32) -> Option<usize> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;

    for (offset, ch) in text.char_indices() {
        if current_line == line {
            break;
        }
        if ch == '\n' {
            current_line += 1;
            line_start = offset + ch.len_utf8();
        }
    }

    if current_line != line {
        return None;
    }

    let line_text = text[line_start..]
        .split_once('\n')
        .map(|(line, _)| line)
        .unwrap_or(&text[line_start..]);
    let mut utf16_units = 0u32;
    for (relative_offset, ch) in line_text.char_indices() {
        if utf16_units == character {
            return Some(line_start + relative_offset);
        }

        let next_utf16_units = utf16_units + ch.len_utf16() as u32;
        if character < next_utf16_units {
            return None;
        }
        utf16_units = next_utf16_units;
    }

    (utf16_units == character).then_some(line_start + line_text.len())
}

/// Rewrite a Corsa diagnostic message with a Vue-flavored hint when the
/// raw TypeScript phrasing has a more actionable Vue interpretation.
///
/// The original wording is preserved as the prefix so the user can still see
/// what TypeScript reported. The added hint points at the most common Vue
/// cause for that error shape.
pub(super) fn rewrite_corsa_message(message: &str) -> String {
    if let Some(prop) = property_does_not_exist_property(message)
        && prop != "value"
    {
        return cstr!(
            "{message}\n\nIf you intended to read the reactive value, try `.value`. (vize/types)"
        )
        .into();
    }
    if message.starts_with("Type 'Ref<") && message.contains("is not assignable to type") {
        return cstr!(
            "{message}\n\nDid you forget `.value`? Vue refs need to be unwrapped in script context. (vize/types)"
        ).into();
    }
    message.to_string()
}

/// Extract the property name from a TS7053/TS2339 "Property 'X' does not
/// exist on type 'Y'" message. Returns `None` for unrelated messages.
fn property_does_not_exist_property(message: &str) -> Option<&str> {
    let head = "Property '";
    let after = message.strip_prefix(head)?;
    let end = after.find('\'')?;
    let rest = &after[end..];
    if !rest.starts_with("' does not exist") {
        return None;
    }
    Some(&after[..end])
}

#[cfg(test)]
mod hint_tests {
    use super::{property_does_not_exist_property, rewrite_corsa_message};

    #[test]
    fn rewrites_property_does_not_exist_with_value_hint() {
        let original = "Property 'toFixed' does not exist on type 'Ref<number>'.";
        let rewritten = rewrite_corsa_message(original);
        assert!(rewritten.contains(original));
        assert!(
            rewritten.contains(".value"),
            "expected a .value hint, got {rewritten:?}"
        );
    }

    #[test]
    fn leaves_known_property_value_alone() {
        // We don't want to suggest `.value` on a `.value` access — that's
        // already what the user wrote.
        let original = "Property 'value' does not exist on type 'unknown'.";
        let rewritten = rewrite_corsa_message(original);
        assert_eq!(rewritten, original);
    }

    #[test]
    fn rewrites_ref_assignment_with_unwrap_hint() {
        let original = "Type 'Ref<number>' is not assignable to type 'number'.";
        let rewritten = rewrite_corsa_message(original);
        assert!(rewritten.contains(original));
        assert!(rewritten.contains("Did you forget `.value`"));
    }

    #[test]
    fn passes_through_unrelated_messages() {
        let original = "Expected 1 argument, but got 0.";
        assert_eq!(rewrite_corsa_message(original), original);
    }

    #[test]
    fn property_extractor_returns_name() {
        assert_eq!(
            property_does_not_exist_property("Property 'foo' does not exist on type 'Bar'."),
            Some("foo")
        );
        assert_eq!(
            property_does_not_exist_property("Cannot find name 'foo'."),
            None
        );
    }
}

fn source_offset_to_position(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut character = 0u32;
    let target = offset.min(source.len());

    for (current, ch) in source.char_indices() {
        if current >= target {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }

    (line, character)
}

#[cfg(test)]
mod tests {
    use super::{line_character_to_byte_offset, source_offset_to_position};

    #[test]
    fn line_character_to_byte_offset_counts_utf16_code_units() {
        let source = "const icon = \"😀\";\nconst message = icon";

        assert_eq!(
            line_character_to_byte_offset(source, 0, 16),
            Some("const icon = \"😀".len())
        );
        assert_eq!(
            line_character_to_byte_offset(source, 1, 6),
            Some(source.find("message").unwrap())
        );
    }

    #[test]
    fn line_character_to_byte_offset_rejects_surrogate_pair_interior() {
        let source = "a😀b";

        assert_eq!(line_character_to_byte_offset(source, 0, 2), None);
    }

    #[test]
    fn source_offset_to_position_counts_utf16_code_units() {
        let source = "const icon = \"😀\"; missing";
        let offset = source.find("missing").unwrap();

        assert_eq!(source_offset_to_position(source, offset), (0, 19));
    }

    /// Issue #752: editor-side virtual TS generation must rewrite `.vue`
    /// import specifiers to `.vue.ts` so the Corsa session can resolve
    /// siblings via the virtual mirror — alias *and* relative specifiers
    /// both get rewritten, mirroring the batch pipeline.
    #[test]
    fn editor_virtual_ts_rewrites_dot_vue_imports() {
        use crate::DiagnosticService;
        use tower_lsp::lsp_types::Url;

        let uri = Url::parse("file:///tmp/Host.vue").expect("parse uri");
        let content = "<script setup lang=\"ts\">\n\
                       import App from './app.vue'\n\
                       import Sibling from '../shared/Sib.vue'\n\
                       import Aliased from '@/Alias.vue'\n\
                       import { ref } from 'vue'\n\
                       const _u = App\n\
                       const _v = Sibling\n\
                       const _w = Aliased\n\
                       const _r = ref(0)\n\
                       </script>\n\
                       <template><div></div></template>";

        let result = DiagnosticService::generate_virtual_ts(&uri, content, false)
            .expect("virtual ts generated");

        assert!(
            !result.code.contains("'./app.vue'"),
            "expected relative .vue import to be rewritten, got:\n{}",
            result.code,
        );
        assert!(
            result.code.contains("'./app.vue.ts'"),
            "expected rewritten relative specifier, got:\n{}",
            result.code,
        );
        assert!(
            result.code.contains("'../shared/Sib.vue.ts'"),
            "expected rewritten parent-path specifier, got:\n{}",
            result.code,
        );
        assert!(
            result.code.contains("'@/Alias.vue.ts'"),
            "expected rewritten alias specifier, got:\n{}",
            result.code,
        );
        // Only relative specifiers feed the sibling overlay; alias and bare
        // imports are excluded since they resolve via tsconfig paths and the
        // ambient stub respectively.
        assert!(
            result.relative_vue_imports.iter().any(|s| s == "./app.vue"),
            "expected ./app.vue in relative_vue_imports, got {:?}",
            result.relative_vue_imports,
        );
        assert!(
            result
                .relative_vue_imports
                .iter()
                .any(|s| s == "../shared/Sib.vue"),
            "expected ../shared/Sib.vue in relative_vue_imports, got {:?}",
            result.relative_vue_imports,
        );
        assert!(
            !result
                .relative_vue_imports
                .iter()
                .any(|s| s == "@/Alias.vue"),
            "alias specifier must not appear in relative_vue_imports",
        );
    }
}
