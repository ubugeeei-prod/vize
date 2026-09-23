//! Rewrite emitted declarations and compose their generated map coordinates.

use super::DECLARATION_HELPERS_FILE;
use super::{declaration_helpers, declaration_maps};
use crate::batch::declaration_path::is_declaration_file;
use crate::batch::{CorsaResult, import_rewriter::ImportRewriter};
use oxc_span::SourceType;
use std::path::Path;
use vize_carton::String;

pub(super) fn rewrite_declaration_outputs(out_dir: &Path) -> CorsaResult<()> {
    let rewriter = ImportRewriter::new();
    if !out_dir.exists() {
        return Ok(());
    }

    let mut wrote_vue_declaration = false;
    for entry in walkdir::WalkDir::new(out_dir) {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !is_declaration_file(path) {
            continue;
        }

        let content = std::fs::read_to_string(path)?;
        let imports = rewriter.rewrite_declaration_specifiers(&content, SourceType::ts());
        let mut rewritten = imports.code;
        let mut removed = Vec::new();
        let mut prefix_len = 0;
        // Generated `.vue.d.ts` outputs reference the hoisted helper type
        // aliases (`__EmitFn`, `__RuntimePropShape`, ...). Wire each one to
        // the helpers declaration shipped alongside the outputs so consumer
        // programs resolve them without including the virtual mirror.
        if name.ends_with(".vue.d.ts") {
            wrote_vue_declaration = true;
            removed = internal_field_ranges(&rewritten);
            rewritten = remove_ranges(&rewritten, &removed);
            let depth = path
                .strip_prefix(out_dir)
                .ok()
                .and_then(|relative| relative.parent())
                .map(|parent| parent.components().count())
                .unwrap_or(0);
            let mut reference = declaration_helpers::import_for(&rewritten, depth);
            prefix_len = reference.len();
            reference.push_str(&rewritten);
            rewritten = reference.as_str().into();
        }
        if rewritten.as_str() != content {
            declaration_maps::rewrite_generated_positions(
                path,
                &content,
                &rewritten,
                &imports.source_map,
                &removed,
                prefix_len,
            )?;
            std::fs::write(path, rewritten.as_str())?;
        }
    }

    if wrote_vue_declaration {
        std::fs::write(
            out_dir.join(DECLARATION_HELPERS_FILE),
            declaration_helpers::module(),
        )?;
    }

    Ok(())
}

#[cfg(test)]
pub(super) fn strip_internal_vue_declaration_fields(source: &str) -> Option<String> {
    let ranges = internal_field_ranges(source);
    (!ranges.is_empty()).then(|| remove_ranges(source, &ranges))
}

fn internal_field_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut skipping = false;
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        if skipping || line.contains("__vizeFallthroughProps") {
            skipping = !line.contains(';');
            ranges.push(offset..offset + line.len());
        } else if line.contains("__vizeHasFallthroughProps")
            || line.contains("__vizeComponentMarker")
        {
            ranges.push(offset..offset + line.len());
        }
        offset += line.len();
    }
    ranges
}

fn remove_ranges(source: &str, ranges: &[std::ops::Range<usize>]) -> String {
    let mut result = String::default();
    let mut cursor = 0;
    for range in ranges {
        result.push_str(source.get(cursor..range.start).unwrap_or_default());
        cursor = range.end;
    }
    result.push_str(source.get(cursor..).unwrap_or_default());
    result
}
