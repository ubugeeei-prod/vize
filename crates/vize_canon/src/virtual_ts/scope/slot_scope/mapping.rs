use vize_croquis::Scope;

use super::super::emit::SlotParameterGeneratedParts;
use crate::virtual_ts::types::VizeMapping;

fn push_mapped_segment(
    mappings: &mut Vec<VizeMapping>,
    generated_start: usize,
    source_start: usize,
    len: usize,
    binding_ranges: &[std::ops::Range<usize>],
) {
    let source_end = source_start + len;
    let mut cursor = source_start;
    for binding in binding_ranges {
        let overlap_start = binding.start.max(source_start);
        let overlap_end = binding.end.min(source_end);
        if overlap_start >= overlap_end {
            continue;
        }
        if cursor < overlap_start {
            let relative = cursor - source_start;
            mappings.push(VizeMapping {
                gen_range: generated_start + relative
                    ..generated_start + relative + overlap_start - cursor,
                src_range: cursor..overlap_start,
                sub_spans: Vec::new(),
            });
        }
        cursor = cursor.max(overlap_end);
    }
    if cursor < source_end {
        let relative = cursor - source_start;
        mappings.push(VizeMapping {
            gen_range: generated_start + relative..generated_start + len,
            src_range: cursor..source_end,
            sub_spans: Vec::new(),
        });
    }
}

pub(super) fn map_slot_props_pattern(
    mappings: &mut Vec<VizeMapping>,
    scope: &Scope,
    pattern_offset: Option<u32>,
    template_offset: u32,
    props_pattern: &str,
    ts: &str,
    generated_parts: SlotParameterGeneratedParts,
) {
    let function_text = ts.get(generated_parts.function_start..).unwrap_or_default();
    let parameter = super::super::emit::slot_parameter(props_pattern);
    let mut binding_ranges = Vec::new();
    for (prop_name, binding) in scope.bindings() {
        let Some(pattern_relative) =
            super::super::emit::pattern_identifier_offset(props_pattern, prop_name)
        else {
            continue;
        };
        let Some(generated_relative) =
            super::super::emit::pattern_identifier_offset(function_text, prop_name)
        else {
            continue;
        };
        let src_start = binding.declaration_offset;
        mappings.push(VizeMapping {
            gen_range: generated_parts.function_start + generated_relative
                ..generated_parts.function_start + generated_relative + prop_name.len(),
            src_range: (template_offset + src_start) as usize
                ..(template_offset + src_start) as usize + prop_name.len(),
            sub_spans: Vec::new(),
        });
        binding_ranges.push(pattern_relative..pattern_relative + prop_name.len());
    }
    // TypeScript anchors a missing renamed property on its key, not on the
    // local binding. Mapping the whole pattern as well as each binding made
    // protocol normalization accept the shorter bindings first and discard
    // the overlapping whole-pattern span, leaving that key unmapped (#12511).
    // Emit only the disjoint gaps around binding identifiers; together with
    // the identifier rows above, every authored byte has exactly one owner.
    if let Some(pattern_offset) = pattern_offset {
        binding_ranges.sort_by_key(|range| (range.start, range.end));
        let authored_start = (template_offset + pattern_offset) as usize;
        let absolute_binding_ranges = binding_ranges
            .iter()
            .map(|range| authored_start + range.start..authored_start + range.end)
            .collect::<Vec<_>>();
        for (part, generated_start) in [
            Some((parameter.pattern, generated_parts.pattern_start)),
            parameter.annotation.zip(generated_parts.annotation_start),
            parameter.default.zip(generated_parts.default_start),
        ]
        .into_iter()
        .flatten()
        .filter(|(part, _)| !part.text.is_empty())
        {
            push_mapped_segment(
                mappings,
                generated_start,
                authored_start + part.start,
                part.text.len(),
                &absolute_binding_ranges,
            );
        }
    }
}
