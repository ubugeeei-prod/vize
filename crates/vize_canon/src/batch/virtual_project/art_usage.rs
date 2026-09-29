use vize_atelier_core::{
    ParserOptions, TemplateSyntaxMode, parser::parse_with_options_and_template_syntax,
};
use vize_atelier_sfc::{SfcDescriptor, script::resolve_template_used_identifiers};
use vize_carton::{Allocator, FxHashSet, String as CompactString};

pub(super) fn collect_art_template_referenced_names(
    descriptor: &SfcDescriptor<'_>,
    template_syntax: TemplateSyntaxMode,
    experimental_in_tag_comments: bool,
) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();

    for block in &descriptor.custom_blocks {
        if block.block_type.as_ref() != "art" {
            continue;
        }
        collect_variant_template_referenced_names(
            &block.content,
            template_syntax,
            experimental_in_tag_comments,
            &mut names,
        );
    }

    names
}

fn collect_variant_template_referenced_names(
    art_content: &str,
    template_syntax: TemplateSyntaxMode,
    experimental_in_tag_comments: bool,
    names: &mut FxHashSet<CompactString>,
) {
    for (start, end) in variant_inner_ranges(art_content) {
        collect_template_source_referenced_names(
            art_content.get(start..end).unwrap_or_default().trim(),
            template_syntax,
            experimental_in_tag_comments,
            names,
        );
    }
}

/// Markup inside `<variant>` in an `<art>` block, as ranges within `art_content`.
fn variant_inner_ranges(art_content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = art_content
        .get(cursor..)
        .and_then(|rest| rest.find("<variant"))
    {
        let start = cursor + relative_start;
        let after_name = start + "<variant".len();
        if !is_variant_tag_boundary(art_content.as_bytes().get(after_name).copied()) {
            cursor = after_name;
            continue;
        }

        let Some(tag_end) = art_content
            .get(start..)
            .and_then(|rest| rest.find('>'))
            .map(|offset| start + offset)
        else {
            break;
        };
        let tag = art_content
            .get(start..=tag_end)
            .unwrap_or_default()
            .trim_end();
        if tag.ends_with("/>") {
            cursor = tag_end + 1;
            continue;
        }

        let template_start = tag_end + 1;
        let Some(close_start) = art_content
            .get(template_start..)
            .and_then(|rest| rest.find("</variant>"))
            .map(|offset| template_start + offset)
        else {
            break;
        };
        if art_content
            .get(template_start..close_start)
            .is_some_and(|inner| !inner.trim().is_empty())
        {
            ranges.push((template_start, close_start));
        }
        cursor = close_start + "</variant>".len();
    }
    ranges
}

/// Full-file template source whose bytes sit at the same offsets as the SFC.
///
/// Variant markup is not a `<template>` block, so its AST spans have to land on
/// the authored file. The returned string is `source.len()` bytes: spaces
/// everywhere except the real template body and each `<variant>` inner. Parsing
/// it with a template offset of 0 type-checks that markup in the same projection
/// as a normal SFC template.
pub(super) fn art_variant_check_source(
    source: &str,
    descriptor: &SfcDescriptor<'_>,
) -> Option<String> {
    let mut ranges = Vec::new();
    for block in &descriptor.custom_blocks {
        if block.block_type.as_ref() != "art" {
            continue;
        }
        let block_start = block.loc.start;
        let block_end = block.loc.end;
        let (body, origin) = match source.get(block_start..block_end) {
            Some(slice) => (slice, block_start),
            None => (block.content.as_ref(), block_start),
        };
        for (start, end) in variant_inner_ranges(body) {
            ranges.push((origin + start, origin + end));
        }
    }
    if ranges.is_empty() {
        return None;
    }

    let mut bytes = vec![b' '; source.len()];
    if let Some(template) = descriptor.template.as_ref() {
        copy_aligned_range(
            &mut bytes,
            source,
            template.loc.start,
            template.loc.end,
            template.content.as_ref(),
        );
    }
    for (start, end) in ranges {
        let Some(text) = source.get(start..end) else {
            continue;
        };
        copy_aligned_range(&mut bytes, source, start, end, text);
    }
    String::from_utf8(bytes).ok()
}

fn copy_aligned_range(dest: &mut [u8], source: &str, start: usize, end: usize, content: &str) {
    let bytes = if source.get(start..end) == Some(content) {
        content.as_bytes()
    } else if end >= start && content.len() == end - start {
        content.as_bytes()
    } else if let Some(slice) = source.get(start..end) {
        slice.as_bytes()
    } else {
        return;
    };
    let Some(slot) = dest.get_mut(start..start + bytes.len()) else {
        return;
    };
    slot.copy_from_slice(bytes);
}

fn is_variant_tag_boundary(next: Option<u8>) -> bool {
    matches!(next, Some(b' ' | b'\t' | b'\n' | b'\r' | b'>') | None)
}

fn collect_template_source_referenced_names(
    template: &str,
    template_syntax: TemplateSyntaxMode,
    experimental_in_tag_comments: bool,
    names: &mut FxHashSet<CompactString>,
) {
    if template.is_empty() {
        return;
    }

    let allocator = Allocator::new();
    let (root, _) = parse_with_options_and_template_syntax(
        &allocator,
        template,
        ParserOptions {
            experimental_in_tag_comments,
            ..ParserOptions::default()
        },
        template_syntax,
    );
    names.extend(resolve_template_used_identifiers(&root).used_ids);
}
