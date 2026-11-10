use vize_croquis::Scope;

use crate::virtual_ts::types::VizeMapping;

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
    let pattern = pattern_offset.and_then(|source_offset| {
        function_text.find(props_pattern).map(|generated_offset| {
            (
                function_gen_start + generated_offset,
                (template_offset + source_offset) as usize,
            )
        })
    });
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
    if let Some((generated_start, source_start)) = pattern {
        binding_ranges.sort_by_key(|range| (range.start, range.end));
        let mut cursor = 0;
        for binding in binding_ranges {
            if cursor < binding.start {
                mappings.push(VizeMapping {
                    gen_range: generated_start + cursor..generated_start + binding.start,
                    src_range: source_start + cursor..source_start + binding.start,
                    sub_spans: Vec::new(),
                });
            }
            cursor = cursor.max(binding.end);
        }
        if cursor < props_pattern.len() {
            mappings.push(VizeMapping {
                gen_range: generated_start + cursor..generated_start + props_pattern.len(),
                src_range: source_start + cursor..source_start + props_pattern.len(),
                sub_spans: Vec::new(),
            });
        }
    }
}
