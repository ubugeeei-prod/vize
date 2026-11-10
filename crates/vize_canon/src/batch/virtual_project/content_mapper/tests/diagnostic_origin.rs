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

/// The authored range of the `occurrence`th `token` after `after`.
fn authored(source: &str, after: &str, token: &str, occurrence: usize) -> Range<usize> {
    let from = source.find(after).expect("authored anchor");
    let start = from
        + source[from..]
            .match_indices(token)
            .nth(occurrence)
            .expect("token")
            .0;
    start..start + token.len()
}

/// Where the `occurrence`th generated `node` (the first `len` bytes of
/// `generated`) reports a diagnostic.
fn generated_origin(source: &str, generated: &str, len: usize, occurrence: usize) -> Range<usize> {
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let start = result
        .text
        .match_indices(generated)
        .nth(occurrence)
        .unwrap_or_else(|| panic!("generated {generated} #{occurrence}:\n{}", result.text))
        .0;
    diagnostic_origin(&result.mappings, start..start + len)
}

fn assert_quoted_diagnostic_origin(source: &str, quoted: &str, occurrence: usize, authored: &str) {
    let expected = self::authored(source, "<", authored, occurrence);
    // TypeScript reports the quoted key node, delimiters included.
    let key = quoted[1..].find('"').expect("closing quote") + 2;
    assert_eq!(
        generated_origin(source, quoted, key, occurrence),
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
    // Both generated owners report on the directive. The upstream mapper host
    // does not deduplicate diagnostics, so the transform suppresses the
    // per-prop echo and keeps this whole-props position as the canonical one.
    let check = "__vize_prop_check_0_modelValue";
    assert_eq!(
        generated_origin(model, check, check.len(), 0),
        authored(model, "<template>", "v-model", 0)
    );
    let key = "\"modelValue\"";
    assert_eq!(
        generated_origin(model, "\"modelValue\": count", key.len(), 0),
        authored(model, "<template>", "v-model", 0)
    );

    // An argument model and an ordinary prop keep their own names.
    let named = r#"<script setup lang="ts">
import Child from './Child.vue'
const title = 1
</script>
<template>
<Child v-model:title="title" :count="title" />
</template>
"#;
    assert_eq!(
        generated_origin(named, "\"title\": title", "\"title\"".len(), 0),
        authored(named, "<template>", "title", 0)
    );
    assert_quoted_diagnostic_origin(named, "\"count\": title", 0, "count");
}

#[test]
fn whole_props_argument_starts_on_the_component_tag() {
    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template><Child label="missing count" /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let call = result
        .text
        .find("as __VCheck0)(")
        .expect("component checker call");
    let opening = call + result.text[call..].find('{').expect("props literal");
    assert_eq!(
        diagnostic_origin(&result.mappings, opening..opening + 1),
        authored(source, "<template>", "Child", 0)
    );
}

/// #11675: TypeScript anchors the listener check at the synthetic `(` of
/// `= ((value: number) => undefined)`. That gap used to resolve to the
/// attribute `=`; the wrapper is a diagnostics-only span on the authored value.
#[test]
fn listener_wrapper_paren_anchors_on_the_authored_value() {
    let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>
<template>
<Child foo-bar="ok" @save-item="(value: number) => undefined" />
</template>
"#;
    let value = "(value: number) => undefined";
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let marker = result
        .text
        .find("= ((value: number) => undefined)")
        .unwrap_or_else(|| panic!("wrapped listener:\n{}", result.text));
    let synthetic = marker + "= ".len();
    assert_eq!(&result.text[synthetic..synthetic + 2], "((");
    let origin = diagnostic_origin(&result.mappings, synthetic..synthetic + 1);
    let authored_value = authored(source, "@save-item", value, 0);
    let equals = authored_value.start - 2;
    assert_eq!(
        &source[equals..equals + 1],
        "=",
        "the column before the value must be the attribute '='"
    );
    assert_ne!(
        origin.start, equals,
        "the listener check must not land on the attribute '='"
    );
    assert_eq!(
        origin, authored_value,
        "the wrapping '(' must report the authored listener, which starts at '('"
    );
}

#[test]
fn split_script_setup_spans_start_at_the_authored_block() {
    let source = r#"<script lang="ts">
export type SearchQuery = { value: string };
</script>

<script setup lang="ts">
const values: any = [];
values.map(it => it);
</script>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Split.vue"), source).expect("transform");
    let generated = result.text.find("it => it").expect("generated parameter");
    let original = source.find("it => it").expect("authored parameter");
    let span = result
        .mappings
        .iter()
        .map(|mapping| mapping.0)
        .find(|span| generated >= span[0] && generated < span[0] + span[1])
        .expect("parameter mapping");

    assert_eq!(span[4], 0, "setup line should remain verbatim: {span:?}");
    assert_eq!(span[2] + generated - span[0], original);
}

#[test]
fn commented_template_binding_members_keep_diagnostic_owners() {
    let source = r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";

const items = ref<{ id: number }[]>([]);
const done = ref(false);
</script>

<template>
  <Child
    v-for="item in items"
    :key="item.id"
    :height="
      done &&
      item.kind === 'a'
        ? 30 /* top */ - 20 /* bottom */
        : undefined
    "
  />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let original = source.find("item.kind").expect("authored member");
    let expected = original..original + "item.kind".len();
    let generated = result.text.match_indices("item.kind").collect::<Vec<_>>();
    assert_eq!(generated.len(), 3, "generated checks:\n{}", result.text);
    for (start, _) in generated {
        assert_eq!(
            diagnostic_origin(&result.mappings, start..start + "item.kind".len()),
            expected,
            "every generated copy must report on the authored member",
        );
    }
}
