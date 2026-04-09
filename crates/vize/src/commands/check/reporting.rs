//! Diagnostic reporting and source-map utilities for the check command.
//!
//! Handles mapping diagnostics from virtual TypeScript positions back to original
//! SFC line/column positions, and provides JSON output structures.

use serde::Serialize;

/// JSON output structure for `--format json`.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonOutput {
    pub files: Vec<JsonFileResult>,
    #[serde(rename = "errorCount")]
    pub error_count: usize,
    #[serde(rename = "fileCount")]
    pub file_count: usize,
}

/// Per-file result in JSON output.
#[derive(Serialize)]
#[allow(clippy::disallowed_types)]
pub(crate) struct JsonFileResult {
    pub file: String,
    #[serde(rename = "virtualTs")]
    pub virtual_ts: String,
    pub diagnostics: Vec<String>,
}

/// Convert a line/column position in the virtual TS to a line/column in the original SFC.
///
/// Steps:
/// 1. Convert virtual TS line/col to byte offset in virtual TS
/// 2. Find matching source mapping
/// 3. Compute byte offset in original SFC
/// 4. Convert SFC byte offset to line/col
pub(crate) fn map_diagnostic_position(
    virtual_ts: &str,
    source_map: &[vize_canon::virtual_ts::VizeMapping],
    original_content: &str,
    vts_line: u32,
    vts_character: u32,
) -> (u32, u32) {
    // Step 1: line/col -> byte offset in virtual TS
    let vts_offset = line_col_to_offset(virtual_ts, vts_line, vts_character);

    // Step 2: Find matching source mapping
    for mapping in source_map {
        if vts_offset >= mapping.gen_range.start && vts_offset < mapping.gen_range.end {
            // Step 3: Compute corresponding offset in original SFC
            let delta = vts_offset - mapping.gen_range.start;
            let src_offset = mapping.src_range.start + delta;
            // Clamp to source range
            let src_offset = src_offset.min(mapping.src_range.end.saturating_sub(1));

            // Step 4: Convert SFC offset to line/col (1-based)
            let (line, col) = offset_to_line_col(original_content, src_offset);
            return (line + 1, col + 1);
        }
    }

    // Fallback: return virtual TS position (1-based)
    (vts_line + 1, vts_character + 1)
}

/// Check if a virtual TS position has a source mapping to user code.
/// Returns false for positions in generated code (compiler macros, type helpers, etc.)
pub(crate) fn has_source_mapping(
    virtual_ts: &str,
    source_map: &[vize_canon::virtual_ts::VizeMapping],
    vts_line: u32,
    vts_character: u32,
) -> bool {
    let vts_offset = line_col_to_offset(virtual_ts, vts_line, vts_character);
    source_map
        .iter()
        .any(|m| vts_offset >= m.gen_range.start && vts_offset < m.gen_range.end)
}

/// Convert line/column (0-based) to byte offset in content.
fn line_col_to_offset(content: &str, line: u32, col: u32) -> usize {
    let mut current_line = 0u32;
    let mut offset = 0usize;

    for (i, ch) in content.char_indices() {
        if current_line == line {
            return i + col as usize;
        }
        if ch == '\n' {
            current_line += 1;
        }
        offset = i + ch.len_utf8();
    }

    offset + col as usize
}

/// Convert byte offset to line/column (0-based) in content.
fn offset_to_line_col(content: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, ch) in content.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::{map_diagnostic_position, offset_to_line_col};
    use vize_atelier_core::parser::parse;
    use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
    use vize_canon::virtual_ts::{generate_virtual_ts_with_offsets, VirtualTsOptions};
    use vize_carton::Bump;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    fn generate_output(
        source: &str,
    ) -> (
        std::string::String,
        Vec<vize_canon::virtual_ts::VizeMapping>,
    ) {
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: "test.vue".into(),
                ..Default::default()
            },
        )
        .expect("SFC should parse");

        let script_setup = descriptor
            .script_setup
            .as_ref()
            .expect("test fixture should have script setup");
        let template = descriptor
            .template
            .as_ref()
            .expect("test fixture should have template");

        let allocator = Bump::new();
        let (root, _) = parse(&allocator, &template.content);

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(&script_setup.content);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(&script_setup.content),
            Some(&root),
            script_setup.loc.start as u32,
            template.loc.start as u32,
            &VirtualTsOptions::default(),
        );

        (output.code.into(), output.mappings)
    }

    #[test]
    fn test_map_diagnostic_position_for_wrapped_template_expression_start() {
        let source = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'

const inputRef = useTemplateRef<HTMLInputElement>('input')
</script>

<template>
  <div :data-active="inputRef && inputRef.focus()"></div>
</template>"#;

        let (virtual_ts, source_map) = generate_output(source);
        let expression = "inputRef && inputRef.focus()";

        let generated_offset = virtual_ts.find(expression).unwrap();
        let (vline, vcol) = offset_to_line_col(&virtual_ts, generated_offset);
        let (line, col) = map_diagnostic_position(&virtual_ts, &source_map, source, vline, vcol);

        let source_offset = source.find(expression).unwrap();
        let (expected_line, expected_col) = offset_to_line_col(source, source_offset);

        assert_eq!((line, col), (expected_line + 1, expected_col + 1));
    }

    #[test]
    fn test_map_diagnostic_position_for_inner_identifier_in_wrapped_template_expression() {
        let source = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'

const inputRef = useTemplateRef<HTMLInputElement>('input')
</script>

<template>
  <div :data-active="inputRef && inputRef.focus()"></div>
</template>"#;

        let (virtual_ts, source_map) = generate_output(source);
        let expression = "inputRef && inputRef.focus()";
        let identifier_offset = "inputRef && ".len();

        let generated_offset = virtual_ts.find(expression).unwrap() + identifier_offset;
        let (vline, vcol) = offset_to_line_col(&virtual_ts, generated_offset);
        let (line, col) = map_diagnostic_position(&virtual_ts, &source_map, source, vline, vcol);

        let source_offset = source.find(expression).unwrap() + identifier_offset;
        let (expected_line, expected_col) = offset_to_line_col(source, source_offset);

        assert_eq!((line, col), (expected_line + 1, expected_col + 1));
    }
}
