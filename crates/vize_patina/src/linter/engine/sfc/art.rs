//! Template rules for Musea `<variant>` markup inside `<art>`.

use vize_atelier_sfc::SfcDescriptor;

use super::super::offset_result;
use super::empty_lint_result;
use crate::linter::config::{LintResult, Linter};
use crate::linter::engine::tag_scan::{find_closing_tag, find_start_tag_end};

impl Linter {
    pub(super) fn append_art_variant_template_diagnostics(
        &self,
        source: &str,
        filename: &str,
        result: &mut LintResult,
    ) {
        if !filename.ends_with(".art.vue") || !source_has_block(source, b"<art") {
            return;
        }
        let Ok(descriptor) = crate::linter::script_rules::parse_sfc_for_lint(source, filename)
        else {
            return;
        };
        let mut extra = empty_lint_result(filename);
        for (start, end) in art_variant_ranges(source, &descriptor) {
            let Some(inner) = source.get(start..end) else {
                continue;
            };
            let mut one = self.lint_sfc_template_source(inner, filename);
            offset_result(&mut one, start as u32);
            extra = Self::merge_lint_results(extra, one);
        }
        if extra.diagnostics.is_empty() {
            return;
        }
        let current = std::mem::replace(result, empty_lint_result(filename));
        *result = Self::merge_lint_results(current, extra);
    }
}

fn art_variant_ranges(source: &str, descriptor: &SfcDescriptor<'_>) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for block in &descriptor.custom_blocks {
        if block.block_type.as_ref() != "art" {
            continue;
        }
        let Some(body) = source.get(block.loc.start..block.loc.end) else {
            continue;
        };
        let origin = block.loc.start;
        for (start, end) in variant_inner_ranges(body) {
            ranges.push((origin + start, origin + end));
        }
    }
    ranges
}

/// Inner markup of each non-empty `<variant>`, as ranges within `art_content`.
fn variant_inner_ranges(art_content: &str) -> Vec<(usize, usize)> {
    let bytes = art_content.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = bytes
        .get(cursor..)
        .and_then(|rest| memchr::memmem::find(rest, b"<variant"))
    {
        let start = cursor + relative;
        let after_name = start + "<variant".len();
        if !is_block_tag_boundary(bytes.get(after_name).copied()) {
            cursor = after_name;
            continue;
        }
        let Some(tag_end) = find_start_tag_end(bytes, start) else {
            break;
        };
        if is_self_closing(bytes, tag_end) {
            cursor = tag_end + 1;
            continue;
        }
        let inner_start = tag_end + 1;
        let Some(close) = find_closing_tag(bytes, b"variant", inner_start) else {
            break;
        };
        if art_content
            .get(inner_start..close)
            .is_some_and(|inner| !inner.trim().is_empty())
        {
            ranges.push((inner_start, close));
        }
        cursor = find_start_tag_end(bytes, close)
            .map(|end| end + 1)
            .unwrap_or(close + "</variant>".len());
    }
    ranges
}

fn source_has_block(source: &str, tag: &[u8]) -> bool {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = bytes
        .get(cursor..)
        .and_then(|rest| memchr::memmem::find(rest, tag))
    {
        let start = cursor + relative;
        let next = bytes.get(start + tag.len()).copied();
        if is_block_tag_boundary(next) {
            return true;
        }
        cursor = start + tag.len();
    }
    false
}

fn is_block_tag_boundary(next: Option<u8>) -> bool {
    matches!(
        next,
        Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/') | None
    )
}

fn is_self_closing(bytes: &[u8], tag_end: usize) -> bool {
    let mut index = tag_end;
    while index > 0 {
        index -= 1;
        match bytes.get(index) {
            Some(b' ' | b'\t' | b'\n' | b'\r') => continue,
            Some(b'/') => return true,
            _ => return false,
        }
    }
    false
}
