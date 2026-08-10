use std::path::Path;

use super::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
    ContentMapperSpanKind, generate_vue_content_mapper_transform,
};

#[test]
fn maps_component_event_navigation_to_the_authored_name() {
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const handler = (value: number) => value;
</script>
<template><Child @sa="handler" /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let original = source.find("@sa").unwrap() + 1;
    let navigation = result.text.find("__vize_events_nav_0.sa").unwrap();
    let generated = navigation + "__vize_events_nav_0.".len();
    let handler = result.text.find("// @sa handler").unwrap();

    assert!(
        navigation < handler,
        "completion projection must win reverse mapping"
    );
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[0] == generated
            && mapping.0[1] == "sa".len()
            && mapping.0[2] == original
            && mapping.0[3] == "sa".len()
            && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
    }));
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[2] == original
            && mapping.0[3] == "sa".len()
            && mapping.0[4] == ContentMapperSpanKind::Atom as usize
            && mapping.0[5] == 0
    }));
}

#[test]
fn resolves_complete_kebab_events_through_the_authored_camel_key() {
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save-item="() => undefined" /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let projection = "__vize_kebab_events_nav_0.saveItem";
    let generated = result.text.find(projection).unwrap() + "__vize_kebab_events_nav_0.".len();
    let original = source.find("@save-item").unwrap() + 1;

    assert!(
        result.mappings.iter().any(|mapping| {
            mapping.0[0] == generated
                && mapping.0[1] == "saveItem".len()
                && mapping.0[2] == original
                && mapping.0[3] == "save-item".len()
                && mapping.0[4] == ContentMapperSpanKind::Atom as usize
                && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
        }),
        "{:#?}",
        result.mappings
    );
}

#[test]
fn maps_authored_event_members_without_overlapping_the_macro_type() {
    let source = r#"<script setup lang="ts">
defineEmits<{ save: [value: number] }>();
</script>
<template><button /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Child.vue"), source).expect("transform");
    let original = source.find("save: [value").unwrap();
    let event_map = result.text.find("type __VizeAuthoredEventMap").unwrap();
    let generated = event_map + result.text[event_map..].find("save:").unwrap();

    assert!(result.mappings.iter().any(|mapping| {
        generated == mapping.0[0]
            && mapping.0[1] == "save".len()
            && original == mapping.0[2]
            && mapping.0[3] == "save".len()
            && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
    }));
}

#[test]
fn retains_model_events_when_removing_duplicate_generated_members() {
    let source = r#"<script setup lang="ts">
defineModel<string>("title");
defineEmits<{ save: [value: number] }>();
</script>
<template><button /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Child.vue"), source).expect("transform");

    assert!(result.text.contains("type __VizeAuthoredEventMap = {"));
    assert!(
        result
            .text
            .contains("\"update:title\": [value: (string) | undefined]"),
        "{}",
        result.text
    );
}

#[test]
fn maps_the_emits_alias_only_when_it_resolves_from_module_scope() {
    let module_scoped = r#"<script setup lang="ts">
import type { Emits as Authored } from "./types";
defineEmits<Authored>();
</script>
<template><button /></template>
"#;
    assert!(
        emits_alias_is_mapped(module_scoped),
        "an imported emits type stays navigable from the module-scope alias"
    );

    // The alias sits outside `__setup`, so it cannot see a setup-scope type.
    // Mapping it would report the synthetic "cannot find name" on the macro.
    let setup_scoped = r#"<script setup lang="ts">
import type { GetFormResult } from "./types";
const definition = { transparent: false };
type WidgetProps = GetFormResult<typeof definition>;
defineEmits<WidgetProps>();
</script>
<template><button /></template>
"#;
    assert!(
        !emits_alias_is_mapped(setup_scoped),
        "a setup-scope emits type must not map onto the authored macro"
    );
}

fn emits_alias_is_mapped(source: &str) -> bool {
    let result =
        generate_vue_content_mapper_transform(Path::new("Child.vue"), source).expect("transform");
    let alias = result.text.find("export type Emits").expect("Emits alias");
    let end = alias + result.text[alias..].find(";\n").expect("alias terminator");
    result
        .mappings
        .iter()
        .any(|mapping| mapping.0[0] >= alias && mapping.0[0] + mapping.0[1] <= end)
}

#[test]
fn preserves_generic_event_maps_and_static_parent_prop_inference() {
    let child = r#"<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ pick: [value: T] }>();
</script>
"#;
    let child =
        generate_vue_content_mapper_transform(Path::new("GenericChild.vue"), child).expect("child");
    for expected in [
        "type __VizeAuthoredEventMap<T extends string = string>",
        "type __VizeStaticEventMap<T extends string = string> = __EmitOptions<Emits<T>>",
        "__vizeResolveEvents?: <T extends string = string>",
        "=> __VizeAuthoredEventMap<T>",
    ] {
        assert!(child.text.contains(expected), "{}", child.text);
    }

    let parent = r#"<script setup lang="ts">
import GenericChild from "./GenericChild.vue";
const handlePick = (value: string) => value;
</script>
<template><GenericChild value="chosen" @pick="handlePick" /></template>
"#;
    let parent =
        generate_vue_content_mapper_transform(Path::new("App.vue"), parent).expect("parent");
    for expected in [
        "__vizeResolveEvents?: infer __F",
        "\"value\": \"chosen\"",
        "void __vize_events_nav_0.pick",
    ] {
        assert!(parent.text.contains(expected), "{}", parent.text);
    }
}

#[test]
fn emits_dynamic_generic_event_navigation_inside_the_v_for_scope() {
    let source = r#"<script setup lang="ts">
import NestedGenericChild from "./NestedGenericChild.vue";
const values = ["nested"] as const;
const handleSelect = (value: "nested") => value;
</script>
<template>
  <NestedGenericChild
    v-for="value in values"
    :key="value"
    :value="value"
    @select="handleSelect"
  />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let resolver = result
        .text
        .find("const __vize_events_resolved_0")
        .expect("event resolver");
    let closure = result.text[..resolver]
        .rfind("// Component props in v-for scope")
        .expect("component prop v-for closure");

    assert!(closure < resolver, "{}", result.text);
    assert!(
        result.text[resolver..].contains("\"value\": value"),
        "{}",
        result.text
    );
    assert_eq!(
        result
            .text
            .matches("const __vize_events_resolved_0")
            .count(),
        1
    );

    let original = source.find("@select").unwrap() + 1;
    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[2] == original
            && mapping.0[3] == "select".len()
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
    }));
}

#[test]
fn emits_dynamic_generic_event_navigation_inside_the_v_slot_scope() {
    let source = r#"<script setup lang="ts">
import SlotGenericChild from "./SlotGenericChild.vue";
import SlotProvider from "./SlotProvider.vue";
const handleActivate = (value: "slot") => value;
</script>
<template>
  <SlotProvider v-slot="{ value }">
    <SlotGenericChild :value="value" @activate="handleActivate" />
  </SlotProvider>
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let resolver = result
        .text
        .find("const __vize_events_resolved_")
        .expect("event resolver");
    let closure = result.text[..resolver]
        .rfind("// Component props in v-slot scope")
        .expect("component prop v-slot closure");

    assert!(closure < resolver, "{}", result.text);
    assert!(
        result.text[resolver..].contains("\"value\": value"),
        "{}",
        result.text
    );
    assert_eq!(
        result.text.matches("const __vize_events_resolved_").count(),
        1
    );
}
