use std::path::Path;

use super::generate_vue_content_mapper_transform;

const BRANCH_IGNORE: &str = "// @ts-ignore Inference-only callback branch.";

#[test]
fn event_navigation_suppresses_its_inference_only_callback_copy() {
    let source = r#"<script setup lang="ts">
import type { VNode } from "vue";
import Foo from "./Foo.vue";

const emit = defineEmits<{
  quit: [];
}>();
const slots = defineSlots<Partial<Record<"a" | "b", () => VNode>>>();
</script>

<template>
  <Foo :bar="(baz) => Boolean(slots[baz])" @quit="emit('quit')" />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("parent.vue"), source).expect("transform");
    let guarded = r#"// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.
    "bar": (baz) => Boolean(slots[baz]),"#;

    assert_eq!(
        result.text.matches(guarded).count(),
        3,
        "every inference-only callback copy must suppress its own line:\n{}",
        result.text
    );
    assert_eq!(
        result
            .text
            .matches(
                "// @ts-ignore Inference-only call; the authored prop checker owns diagnostics."
            )
            .count(),
        1,
        "the outer call still suppresses aggregate assignability errors:\n{}",
        result.text
    );
    assert_eq!(
        result
            .text
            .matches("= (baz) => Boolean(slots[baz]);")
            .count(),
        1,
        "the mapped authored check remains the sole unsuppressed diagnostic owner:\n{}",
        result.text
    );
}

#[test]
fn nested_function_type_arrow_keeps_the_outer_parameter_on_the_ignored_line() {
    let source = r#"<script setup lang="ts">
import Foo from "./Foo.vue";

const emit = defineEmits<{
  quit: [];
}>();
</script>

<template>
  <Foo
    :apply="(fn: (value: 'a' | 'b') => boolean,
      value) => fn(value)"
    @quit="emit('quit')"
  />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("parent.vue"), source).expect("transform");
    let comment =
        "// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.";
    let guarded = "\"apply\": (fn: (value: 'a' | 'b') => boolean,       value) => fn(value),";
    let lines: Vec<_> = result.text.lines().collect();
    let mut generated_copies: Vec<_> = lines
        .windows(3)
        .filter_map(|lines| {
            (lines[1].trim_start() == comment)
                .then_some((lines[0].trim_start(), lines[2].trim_start()))
        })
        .collect();
    generated_copies.sort_unstable();

    assert_eq!(
        generated_copies,
        vec![
            (
                "const __vize_emit_props_0 = (undefined as unknown as __Foo_0_emit_resolver)(Foo)({",
                guarded,
            ),
            (
                "const __vize_events_resolved_0 = (undefined as unknown as __vize_events_resolver_0)({",
                guarded,
            ),
            (
                "const __vize_resolved_Foo_props_0 = (undefined as unknown as $ResolvedContextProps)((undefined as unknown as $PropsResolver<typeof Foo>)({",
                guarded,
            ),
        ],
        "every inference-only copy must select the outer arrow and keep its parameter on the ignored line:\n{}",
        result.text
    );
    let authored_owner = "(fn: (value: 'a' | 'b') => boolean,\n      value) => fn(value);";
    assert_eq!(
        result.text.matches(authored_owner).count(),
        1,
        "the authored mapped owner remains the sole unsuppressed copy:\n{}",
        result.text
    );
}

#[test]
fn generic_callback_branches_keep_every_inference_parameter_suppressed() {
    let source = r#"<script setup lang="ts">
import Foo from "./Foo.vue";

const condition = true;
const emit = defineEmits<{ quit: [] }>();
</script>

<template>
  <Foo
    :apply="(void 0, <T,>(
      value) => value)"
    @quit="emit('quit')"
  />
  <Foo
    :apply="(
      <T,>(
        value) => value
    )"
    @quit="emit('quit')"
  />
  <Foo
    :apply="condition
      ? <T,>(
          value) => {
          return value
        }
      : <U,>(
          value) => value"
    @quit="emit('quit')"
  />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("parent.vue"), source).expect("transform");

    let sequence = concat!(
        "\"apply\": (void 0, // @ts-ignore Inference-only callback branch.\n",
        "<T,>(       value) => value),",
    );
    let wrapped = concat!(
        "\"apply\": (\n      // @ts-ignore Inference-only callback branch.\n",
        "<T,>(         value) => value\n    ),",
    );
    let conditional = concat!(
        "\"apply\": condition\n      ? // @ts-ignore Inference-only callback branch.\n",
        "<T,>(           value) => {\n",
        "          return value\n",
        "        }\n",
        "      : // @ts-ignore Inference-only callback branch.\n",
        "<U,>(           value) => value,",
    );
    for expected in [sequence, wrapped, conditional] {
        assert_eq!(
            result.text.matches(expected).count(),
            3,
            "all inference paths must preserve this exact callback shape:\n{expected}\n{}",
            result.text
        );
    }
    assert_eq!(
        result.text.matches(BRANCH_IGNORE).count(),
        12,
        "{}",
        result.text
    );

    let directive_ranges: Vec<_> = result
        .text
        .match_indices(BRANCH_IGNORE)
        .map(|(start, _)| start..start + BRANCH_IGNORE.len())
        .collect();
    assert_eq!(directive_ranges.len(), 12);
    for directive in directive_ranges {
        let feature_masks: Vec<_> = result
            .mappings
            .iter()
            .filter_map(|mapping| {
                let generated = mapping.0[0]..mapping.0[0] + mapping.0[1];
                (generated.start < directive.end && directive.start < generated.end)
                    .then_some(mapping.0[5])
            })
            .collect();
        assert_eq!(
            feature_masks,
            Vec::<usize>::new(),
            "synthetic branch directives must have exactly zero source-map overlaps: {directive:?}\n{:#?}",
            result
                .mappings
                .iter()
                .filter(|mapping| {
                    let generated = mapping.0[0]..mapping.0[0] + mapping.0[1];
                    generated.start < directive.end && directive.start < generated.end
                })
                .collect::<Vec<_>>(),
        );
    }

    for authored_owner in [
        "(void 0, <T,>(\n      value) => value);",
        "(\n      <T,>(\n        value) => value\n    );",
        concat!(
            "condition\n      ? <T,>(\n          value) => {\n",
            "          return value\n        }\n",
            "      : <U,>(\n          value) => value;",
        ),
    ] {
        assert_eq!(
            result.text.matches(authored_owner).count(),
            1,
            "the authored mapped owner must remain byte-identical:\n{authored_owner}\n{}",
            result.text
        );
    }
}
