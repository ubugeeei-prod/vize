//! Byte-stable rewrites of generated content-mapper text.

use std::ops::Range;

use vize_l0::String as CompactString;

use crate::virtual_ts::VizeMapping;

const PRIVATE_MODULE: &str = "vize:virtual-ts";
const PUBLISHED_MODULE: &str = "vize/virtual-ts";

pub(super) fn publish_content_mapper_text(
    code: &mut CompactString,
    mappings: &[VizeMapping],
    share_virtual_module: bool,
) {
    publicize_generated_diagnostic_helpers(code, mappings);
    if !share_virtual_module {
        return;
    }
    debug_assert_eq!(PRIVATE_MODULE.len(), PUBLISHED_MODULE.len());
    let starts = code
        .match_indices(PRIVATE_MODULE)
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    for start in starts.into_iter().rev() {
        let end = start + PRIVATE_MODULE.len();
        code.replace_range(start..end, PUBLISHED_MODULE);
    }
}

/// Give generated component-input helpers public display names before the
/// upstream host parses the virtual module. The replacements are byte-identical
/// so every projection offset stays stable.
///
/// Script `/// <reference types>` values are copied into the generated prefix
/// without mappings. Treat those complete lines as authored ranges as well, so
/// a package that happens to use an internal helper name is never rewritten.
fn publicize_generated_diagnostic_helpers(code: &mut CompactString, mappings: &[VizeMapping]) {
    let mut ranges = mappings
        .iter()
        .map(|mapping| mapping.gen_range.clone())
        .collect::<Vec<_>>();
    ranges.extend(reference_type_directive_ranges(code));
    ranges.sort_unstable_by_key(|range| range.start);
    let mut authored_ranges: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(last) = authored_ranges.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
        } else {
            authored_ranges.push(range);
        }
    }

    for (internal, display) in [
        ("__VizeComponentCheckProps", "ComponentPropsForTemplate"),
        ("__VizePublicComponentAttrs", "VueTemplatePublicAttrsType"),
    ] {
        debug_assert_eq!(internal.len(), display.len());
        let occurrences = code
            .match_indices(internal)
            .map(|(start, _)| start)
            .collect::<Vec<_>>();
        for start in occurrences {
            let end = start + internal.len();
            let candidate = start..end;
            let next = authored_ranges.partition_point(|range| range.end <= candidate.start);
            let authored = authored_ranges
                .get(next)
                .is_some_and(|range| candidate.start < range.end && range.start < candidate.end);
            if !authored {
                code.replace_range(candidate, display);
            }
        }
    }
}

fn reference_type_directive_ranges(code: &str) -> Vec<Range<usize>> {
    let mut start = 0;
    code.split_inclusive('\n')
        .filter_map(|line| {
            let range = start..start + line.len();
            start = range.end;
            line.trim_start()
                .starts_with("/// <reference types=")
                .then_some(range)
        })
        .collect()
}
