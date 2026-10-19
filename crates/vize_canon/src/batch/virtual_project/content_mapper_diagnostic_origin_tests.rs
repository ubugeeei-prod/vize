//! Diagnostic starts as TypeScript's content-mapper host maps them (#9753).

use std::ops::Range;
use std::path::Path;

use crate::batch::{ContentMapperSpan, generate_vue_content_mapper_transform};

/// `spanmap.VirtualToOriginalSpan`: a boundary inside a segment maps through
/// it, and a boundary in a gap maps to the preceding segment's original end.
fn diagnostic_origin(spans: &[ContentMapperSpan], virtual_range: Range<usize>) -> Range<usize> {
    let segment_at = |position: usize| {
        spans
            .iter()
            .rposition(|span| span.0[0] <= position)
            .map(|index| (index, position < spans[index].0[0] + spans[index].0[1]))
    };
    let map = |position: usize, high: bool| match segment_at(position) {
        None => 0,
        Some((index, false)) => spans[index].0[2] + spans[index].0[3],
        Some((index, true)) => {
            let [virtual_start, _, original_start, original_length, kind, _] = spans[index].0;
            match (kind, high) {
                (0, _) => original_start + (position - virtual_start) + usize::from(high),
                (_, false) => original_start,
                (_, true) => original_start + original_length,
            }
        }
    };
    map(virtual_range.start, false)..map(virtual_range.end - 1, true)
}

fn assert_quoted_diagnostic_origin(source: &str, quoted: &str, occurrence: usize, authored: &str) {
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let start = result
        .text
        .match_indices(quoted)
        .nth(occurrence)
        .unwrap_or_else(|| panic!("generated {quoted} #{occurrence}:\n{}", result.text))
        .0;
    let expected = source
        .match_indices(authored)
        .nth(occurrence)
        .map(|(start, text)| start..start + text.len())
        .expect("authored token");
    // TypeScript reports the quoted key node, delimiters included.
    let key = quoted[1..].find('"').expect("closing quote") + 2;
    assert_eq!(
        diagnostic_origin(&result.mappings, start..start + key),
        expected,
        "{quoted} #{occurrence} must report on authored {authored:?}"
    );
}

#[test]
fn duplicate_css_module_names_report_on_each_module_value() {
    let source = "<template><div /></template>\n<style module=\"foo\">.same {}</style>\n<style module=\"foo\">.same {}</style>\n";
    for occurrence in 0..2 {
        assert_quoted_diagnostic_origin(source, "\"foo\":", occurrence, "foo");
    }
}

#[test]
fn quoted_component_prop_keys_report_on_the_authored_attribute() {
    let unknown = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template>
<Child :count="1" :notHere="true" />
</template>
"#;
    assert_quoted_diagnostic_origin(unknown, "\"notHere\": true", 0, "notHere");

    let model = r#"<script setup lang="ts">
import { ref } from 'vue'
import Child from './Child.vue'
const count = ref('wrong')
</script>
<template>
<Child v-model="count" />
</template>
"#;
    assert_quoted_diagnostic_origin(model, "\"modelValue\": count", 0, "v-model");
}
