use vize_croquis::Scope;

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
    function_gen_start: usize,
    ts: &str,
) {
    let function_text = ts.get(function_gen_start..).unwrap_or_default();
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
            gen_range: function_gen_start + generated_relative
                ..function_gen_start + generated_relative + prop_name.len(),
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
        let mut generated_cursor = function_text.find('(').map_or(0, |at| at + 1);
        for part in [
            Some(parameter.pattern),
            parameter.annotation,
            parameter.default,
        ]
        .into_iter()
        .flatten()
        .filter(|part| !part.text.is_empty())
        {
            let Some(relative) = function_text
                .get(generated_cursor..)
                .and_then(|text| text.find(part.text))
                .map(|relative| generated_cursor + relative)
            else {
                continue;
            };
            push_mapped_segment(
                mappings,
                function_gen_start + relative,
                authored_start + part.start,
                part.text.len(),
                &absolute_binding_ranges,
            );
            generated_cursor = relative + part.text.len();
        }
    }
}
