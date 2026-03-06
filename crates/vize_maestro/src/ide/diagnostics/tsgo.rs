//! tsgo (TypeScript Go) integration for collecting TypeScript diagnostics.
//!
//! This module generates virtual TypeScript from Vue SFCs and uses the tsgo
//! LSP bridge to collect type-checking diagnostics.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};

use crate::server::ServerState;

use super::{DiagnosticService, SourceMapping, VirtualTsResult};
use vize_carton::cstr;

impl DiagnosticService {
    /// Collect diagnostics from tsgo LSP.
    pub(super) async fn collect_tsgo_diagnostics(
        state: &ServerState,
        uri: &Url,
    ) -> Vec<Diagnostic> {
        tracing::info!("collect_tsgo_diagnostics: {}", uri);

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

        // Get tsgo bridge
        tracing::info!("getting tsgo bridge...");
        let Some(bridge) = state.get_tsgo_bridge().await else {
            tracing::warn!("tsgo bridge not available");
            return vec![];
        };
        tracing::info!("tsgo bridge acquired");

        // Generate virtual TypeScript
        let is_art_file = uri.path().ends_with(".art.vue");
        let virtual_result = if is_art_file {
            Self::generate_virtual_ts_for_art(uri, &content)
        } else {
            Self::generate_virtual_ts(uri, &content)
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
        tracing::info!(
            "generated virtual ts ({} bytes), user_code_start={}, sfc_script_start={}, template_scope_start={}, mappings_count={}",
            virtual_ts.len(),
            user_code_start_line,
            sfc_script_start_line,
            template_scope_start_line,
            line_mappings.iter().filter(|m| m.is_some()).count()
        );

        // Create virtual document name (used by tsgo bridge to create the full URI)
        let virtual_name = cstr!("{}.ts", uri.path());

        // Open or update document in tsgo (uses didChange if already open)
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

        // Get diagnostics (will poll for publishDiagnostics notification)
        tracing::info!(
            "waiting for diagnostics from tsgo bridge for {}",
            virtual_uri
        );
        let Ok(tsgo_diags) = bridge.get_diagnostics(&virtual_uri).await else {
            tracing::warn!("failed to get diagnostics from tsgo");
            return vec![];
        };

        tracing::info!(
            "tsgo returned {} raw diagnostics for {}",
            tsgo_diags.len(),
            virtual_uri
        );

        // Log each diagnostic for debugging
        for (i, diag) in tsgo_diags.iter().enumerate() {
            tracing::info!(
                "  raw diag[{}]: line {}-{}, message: {}",
                i,
                diag.range.start.line,
                diag.range.end.line,
                &diag.message[..diag.message.len().min(100)]
            );
        }

        // Helper to convert byte offset to (line, column) - both 0-indexed
        let offset_to_position = |offset: u32| -> (u32, u32) {
            let mut line = 0u32;
            let mut col = 0u32;
            let mut current = 0u32;

            for ch in content.chars() {
                if current >= offset {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
                current += ch.len_utf8() as u32;
            }

            (line, col)
        };

        // Convert to LSP diagnostics with proper position mapping
        tsgo_diags
            .into_iter()
            .filter_map(|diag| {
                // Skip diagnostics in preamble (before user script content)
                if diag.range.start.line < user_code_start_line {
                    tracing::debug!(
                        "skipping preamble diagnostic at line {} (user code starts at {}): {}",
                        diag.range.start.line,
                        user_code_start_line,
                        &diag.message[..diag.message.len().min(50)]
                    );
                    return None;
                }

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

                // Determine if this is a script error or template error
                let is_template_error = diag.range.start.line >= template_scope_start_line;

                let (start_line, end_line, start_char, end_char) = if is_template_error {
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
                        let (start_line, start_col) = offset_to_position(src_mapping.start);
                        let (end_line, end_col) = offset_to_position(src_mapping.end);

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
                    // Script error - map using user code offset
                    let user_code_offset = diag.range.start.line.saturating_sub(user_code_start_line);
                    let user_code_offset_end = diag.range.end.line.saturating_sub(user_code_start_line);

                    // sfc_script_start_line is 1-indexed, convert to 0-indexed
                    // Add skipped_import_lines to account for import lines that were moved to module scope
                    let skipped_lines = virtual_result.skipped_import_lines;
                    let start = (sfc_script_start_line.saturating_sub(1)) + user_code_offset + skipped_lines;
                    let end = (sfc_script_start_line.saturating_sub(1)) + user_code_offset_end + skipped_lines;

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
                    source: Some("vize/tsgo".to_string()),
                    message: diag.message,
                    ..Default::default()
                })
            })
            .collect()
    }

    /// Generate virtual TypeScript for a Vue SFC.
    pub(super) fn generate_virtual_ts(uri: &Url, content: &str) -> Option<VirtualTsResult> {
        use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
        use vize_canon::virtual_ts::generate_virtual_ts;
        use vize_croquis::{Analyzer, AnalyzerOptions};

        let options = SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = parse_sfc(content, options).ok()?;

        // Get script block info
        let (script_content, sfc_script_start_line) = descriptor
            .script_setup
            .as_ref()
            .map(|s| (s.content.as_ref(), s.loc.start_line as u32))
            .or_else(|| {
                descriptor
                    .script
                    .as_ref()
                    .map(|s| (s.content.as_ref(), s.loc.start_line as u32))
            })?;

        let template_block = descriptor.template.as_ref()?;
        let template_offset = template_block.loc.start as u32;

        let allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&allocator, &template_block.content);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script(script_content);
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        let output = generate_virtual_ts(
            &summary,
            Some(script_content),
            Some(&template_ast),
            template_offset,
        );
        let code = output.code;

        // Count import lines in script content (these are moved to module scope)
        // Import lines are skipped from user setup code section
        let skipped_import_lines = Self::count_import_lines(script_content);

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
        let line_mappings = Self::parse_vize_map_comments(&code);

        Some(VirtualTsResult {
            #[allow(clippy::disallowed_methods)]
            code: code.to_string(),
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
        use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
        use vize_canon::virtual_ts::generate_virtual_ts;
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
        let variant = art_desc.default_variant()?;
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

        // Get script block info
        let (script_content, sfc_script_start_line) = descriptor
            .script_setup
            .as_ref()
            .map(|s| (s.content.as_ref(), s.loc.start_line as u32))
            .or_else(|| {
                descriptor
                    .script
                    .as_ref()
                    .map(|s| (s.content.as_ref(), s.loc.start_line as u32))
            })?;

        // Parse template AST
        let template_allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&template_allocator, template_content);

        // Analyze script + template
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script(script_content);
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        let output = generate_virtual_ts(
            &summary,
            Some(script_content),
            Some(&template_ast),
            template_offset,
        );
        let code = output.code;

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

        Some(VirtualTsResult {
            #[allow(clippy::disallowed_methods)]
            code: code.to_string(),
            user_code_start_line,
            sfc_script_start_line,
            template_scope_start_line,
            line_mappings,
            skipped_import_lines,
        })
    }
}
