//! Rewrites of generated content-mapper text.

use std::ops::Range;

use vize_l0::String as CompactString;

use crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector;
use crate::virtual_ts::{VizeMapping, VizeSemanticLink};

use super::super::vue_codegen::generic_attribute::GenericCopies;

const PRIVATE_MODULE: &str = "vize:virtual-ts";
const PUBLISHED_MODULE: &str = "vize/virtual-ts";
const TYPESCRIPT_NOCHECK_DIRECTIVE: &str = "// @ts-nocheck\n";
const REFERENCE_TYPES_DIRECTIVE: &str = "/// <reference types=";

pub(super) fn publish_content_mapper_text(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
    share_virtual_module: bool,
) {
    publicize_generated_diagnostic_helpers(
        code,
        mappings,
        semantic_links,
        prop_default_key_links,
        generic_copies,
        diagnostic_ownership,
    );
    if !share_virtual_module {
        return;
    }
    debug_assert_eq!(PRIVATE_MODULE.len(), PUBLISHED_MODULE.len());
    let starts = memchr::memmem::find_iter(code.as_bytes(), PRIVATE_MODULE).collect::<Vec<_>>();
    for start in starts.into_iter().rev() {
        let end = start + PRIVATE_MODULE.len();
        code.replace_range(start..end, PUBLISHED_MODULE);
    }
}

pub(super) fn prepend_typescript_nocheck(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
) {
    let offset = TYPESCRIPT_NOCHECK_DIRECTIVE.len();
    let mut prefixed = CompactString::with_capacity(offset + code.len());
    prefixed.push_str(TYPESCRIPT_NOCHECK_DIRECTIVE);
    prefixed.push_str(code.as_str());
    *code = prefixed;
    for mapping in mappings {
        mapping.gen_range.start += offset;
        mapping.gen_range.end += offset;
        for sub_span in &mut mapping.sub_spans {
            sub_span.gen_range.start += offset;
            sub_span.gen_range.end += offset;
        }
    }
    for link in semantic_links.iter_mut().chain(prop_default_key_links) {
        link.shift_generated_ranges(offset);
    }
}

/// Give generated component-input helpers public display names before the
/// upstream host parses the virtual module.
///
/// `$ComponentCheckProps` and `$PublicComponentAttrs` are shorter than the
/// display names. Each replacement records that growth so later generated
/// coordinates stay on the same text. Script `/// <reference types>` values
/// are copied into the generated prefix without mappings. Treat those complete
/// lines as authored ranges as well, so a package that happens to use an
/// internal helper name is never rewritten.
fn publicize_generated_diagnostic_helpers(
    code: &mut CompactString,
    mappings: &mut [VizeMapping],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    mut generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
) {
    let mut authored_ranges = mappings
        .iter()
        .map(|mapping| mapping.gen_range.clone())
        .collect::<Vec<_>>();
    authored_ranges.extend(reference_type_directive_ranges(code));
    authored_ranges.sort_unstable_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for range in authored_ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
        } else {
            merged.push(range);
        }
    }

    for (internal, display) in [
        ("$ComponentCheckProps", "ComponentPropsForTemplate"),
        ("$PublicComponentAttrs", "VueTemplatePublicAttrsType"),
    ] {
        debug_assert!(display.len() > internal.len());
        let occurrences = memchr::memmem::find_iter(code.as_bytes(), internal).collect::<Vec<_>>();
        for start in occurrences.into_iter().rev() {
            let old_len = internal.len();
            let end = start + old_len;
            let candidate = start..end;
            let next = merged.partition_point(|range| range.end <= candidate.start);
            let authored = merged
                .get(next)
                .is_some_and(|range| candidate.start < range.end && range.start < candidate.end);
            if authored {
                continue;
            }
            let new_len = display.len();
            code.replace_range(candidate, display);
            note_public_name_growth(
                mappings,
                &mut merged,
                semantic_links,
                prop_default_key_links,
                generic_copies.as_deref_mut(),
                diagnostic_ownership,
                start,
                old_len,
                new_len,
            );
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "all generated coordinate families must shift in one atomic operation"
)]
fn note_public_name_growth(
    mappings: &mut [VizeMapping],
    authored_ranges: &mut [Range<usize>],
    semantic_links: &mut [VizeSemanticLink],
    prop_default_key_links: &mut [VizeSemanticLink],
    generic_copies: Option<&mut GenericCopies>,
    diagnostic_ownership: Option<&DiagnosticOwnershipCollector>,
    start: usize,
    old_len: usize,
    new_len: usize,
) {
    let old_end = start + old_len;
    let extra = new_len - old_len;
    for mapping in mappings {
        shift_generated_range(&mut mapping.gen_range, start, old_end, extra);
        for sub_span in &mut mapping.sub_spans {
            shift_generated_range(&mut sub_span.gen_range, start, old_end, extra);
        }
    }
    for range in authored_ranges {
        shift_generated_range(range, start, old_end, extra);
    }
    for link in semantic_links.iter_mut().chain(prop_default_key_links) {
        shift_generated_range(&mut link.source_range, start, old_end, extra);
        shift_generated_range(&mut link.target_range, start, old_end, extra);
    }
    if let Some(copies) = generic_copies {
        copies.note_generated_replacement(start, old_len, new_len);
    }
    if let Some(collector) = diagnostic_ownership {
        collector
            .borrow_mut()
            .note_generated_replacement(start, old_len, new_len);
    }
}

fn shift_generated_range(range: &mut Range<usize>, start: usize, old_end: usize, extra: usize) {
    if range.start >= old_end {
        range.start += extra;
        range.end += extra;
    } else if range.end > start && range.end >= old_end {
        range.end += extra;
    }
}

/// Complete lines whose first non-whitespace text is a types reference.
fn reference_type_directive_ranges(code: &str) -> Vec<Range<usize>> {
    let bytes = code.as_bytes();
    memchr::memmem::find_iter(bytes, REFERENCE_TYPES_DIRECTIVE)
        .filter_map(|at| {
            let start = memchr::memrchr(b'\n', bytes.get(..at)?).map_or(0, |newline| newline + 1);
            code.get(start..at)?
                .chars()
                .all(char::is_whitespace)
                .then(|| {
                    let end = bytes
                        .get(at..)
                        .and_then(|rest| memchr::memchr(b'\n', rest))
                        .map_or(code.len(), |newline| at + newline + 1);
                    start..end
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::reference_type_directive_ranges;

    #[test]
    fn reference_directive_lines_match_a_line_scan() {
        let code = "/// <reference types=\"a\" />\n  /// <reference types=\"b\" />\r\nx /// <reference types=\"c\" />\n\u{3000}/// <reference types=\"d\" /> /// <reference types=\"e\" />\n/// <reference path=\"f\" />\n\t/// <reference types=\"g\" />";
        let mut start = 0;
        let expected = code
            .split_inclusive('\n')
            .filter_map(|line| {
                let range = start..start + line.len();
                start = range.end;
                line.trim_start()
                    .starts_with("/// <reference types=")
                    .then_some(range)
            })
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 4);
        assert_eq!(reference_type_directive_ranges(code), expected);
    }
}
