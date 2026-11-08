#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use std::path::Path;

use super::super::span_features::{
    CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER, CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_USE,
};
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
                && mapping.0[4] == ContentMapperSpanKind::Alias as usize
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
    let event_map = result.text.find("type $AuthoredEventMap").unwrap();
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

    assert!(
        result
            .text
            .contains("type $AuthoredEventMap = $StaticEventMap & {")
    );
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
        "type $AuthoredEventMap<T extends string = string>",
        "type $StaticEventMap<T extends string = string> = __EmitOptions<Emits<T>>",
        "__vizeResolveEvents?: <T extends string = string>",
        "=> $AuthoredEventMap<T>",
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

/// A template use of a shadowed setup binding names the authored binding for
/// navigation and a const, unwrapped proxy for hover (#11636, #11638).
#[test]
fn template_uses_of_shadowed_setup_bindings_navigate_to_the_authored_binding() {
    let source = r#"<script setup lang="ts">
import { ref } from "vue";
const open = ref(false);
const items = ref([1]);
function toggle() {}
</script>
<template>
  <p v-if="open" @click="open = false">{{ open }}</p>
  <button @click="toggle" />
  <i v-for="open in items" :key="open">{{ open }}</i>
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let template = source.find("<template>").unwrap();
    let loop_start = source.find("<i v-for").unwrap();
    let uses = |name: &str| {
        source
            .match_indices(name)
            .map(|(offset, _)| offset)
            .filter(|offset| *offset > template)
            .collect::<Vec<_>>()
    };
    let projected = |offset: usize, len: usize, features: usize, prefix: &str| {
        result.mappings.iter().any(|mapping| {
            let [
                generated,
                generated_len,
                original,
                original_len,
                kind,
                mapping_features,
            ] = mapping.0;
            let line = result.text[..generated]
                .rsplit_once('\n')
                .map_or("", |(_, line)| line);
            original == offset
                && original_len == len
                && generated_len == len
                && kind == ContentMapperSpanKind::Verbatim as usize
                && mapping_features == features
                && line.trim_start() == prefix
        })
    };
    let hover_owner_count = |offset: usize, len: usize| {
        result
            .mappings
            .iter()
            .filter(|mapping| {
                mapping.0[2] == offset
                    && mapping.0[3] == len
                    && mapping.0[5] & CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER != 0
            })
            .count()
    };
    let (setup, alias): (Vec<_>, Vec<_>) = uses("open")
        .into_iter()
        .partition(|offset| *offset < loop_start);
    assert_eq!((setup.len(), alias.len()), (3, 3));
    for offset in &setup {
        for (features, prefix) in [
            (
                CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_USE,
                crate::virtual_ts::generated_names::TEMPLATE_USE_PREFIX,
            ),
            (
                CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER,
                crate::virtual_ts::generated_names::TEMPLATE_HOVER_PREFIX,
            ),
        ] {
            assert!(
                projected(*offset, "open".len(), features, prefix),
                "{offset}:\n{}",
                result.text
            );
        }
        assert_eq!(hover_owner_count(*offset, "open".len()), 1);
    }
    // The `v-for` source reads `items` in the scope around the loop.
    for offset in uses("items") {
        assert!(
            projected(
                offset,
                "items".len(),
                CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_USE,
                crate::virtual_ts::generated_names::TEMPLATE_USE_PREFIX,
            ),
            "{offset}:\n{}",
            result.text
        );
        assert!(
            projected(
                offset,
                "items".len(),
                CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER,
                crate::virtual_ts::generated_names::TEMPLATE_HOVER_PREFIX,
            ),
            "{offset}:\n{}",
            result.text
        );
        assert_eq!(hover_owner_count(offset, "items".len()), 1);
    }
    // A `v-for` alias and an unshadowed function are already the binding TS sees.
    for offset in alias.into_iter().chain(uses("toggle")) {
        assert!(
            !result.mappings.iter().any(|mapping| {
                mapping.0[2] == offset
                    && matches!(
                        mapping.0[5],
                        CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_USE
                            | CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER
                    )
            }),
            "{offset}:\n{}",
            result.text
        );
    }
}

#[test]
fn template_use_offsets_project_decoded_entities_back_to_authored_bytes() {
    let source = r#"<script setup lang="ts">
import { ref } from "vue";
const open = ref(false);
const rows = ref([1]);
</script>
<template>
  {{ /* 🌸 */ 0 &lt; open }}
  <p :title="'é' + (0 &lt; open ? 'yes' : 'no')" />
  <i v-for="item in (0 &lt; rows.length ? rows : [])">{{ item }}</i>
</template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("Entities.vue"), source)
        .expect("transform");
    let template = source.find("<template>").unwrap();

    for name in ["open", "rows"] {
        let uses = source
            .match_indices(name)
            .map(|(offset, _)| offset)
            .filter(|offset| *offset > template);
        for offset in uses {
            for (features, prefix) in [
                (
                    CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_USE,
                    crate::virtual_ts::generated_names::TEMPLATE_USE_PREFIX,
                ),
                (
                    CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_HOVER,
                    crate::virtual_ts::generated_names::TEMPLATE_HOVER_PREFIX,
                ),
            ] {
                assert!(
                    result.mappings.iter().any(|mapping| {
                        let generated = mapping.0[0];
                        let line = result.text[..generated]
                            .rsplit_once('\n')
                            .map_or("", |(_, line)| line);
                        mapping.0[1] == name.len()
                            && mapping.0[2] == offset
                            && mapping.0[3] == name.len()
                            && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
                            && mapping.0[5] == features
                            && line.trim_start() == prefix
                    }),
                    "{name} at {offset}:\n{}",
                    result.text
                );
            }
        }
    }
}

#[test]
fn options_api_setup_properties_anchor_template_navigation_without_a_lexical_local() {
    let source = r#"<script lang="ts">
import { ref } from "vue";
export default {
  setup() {
    return { ready: ref(true) };
  },
};
</script>
<template>{{ ready }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Options.vue"), source).expect("transform");
    let property = source.find("ready: ref").unwrap();
    let template_use = source.rfind("ready").unwrap();
    let anchor = result
        .text
        .find("const ready: __R_ready = (void 0) as any;")
        .expect("Options API navigation anchor")
        + "const ".len();

    assert!(result.mappings.iter().any(|mapping| {
        mapping.0[0] == anchor
            && mapping.0[1] == "ready".len()
            && mapping.0[2] == property
            && mapping.0[3] == "ready".len()
    }));
    assert!(result.mappings.iter().any(|mapping| {
        let generated = mapping.0[0];
        let line = result.text[..generated]
            .rsplit_once('\n')
            .map_or("", |(_, line)| line);
        mapping.0[2] == template_use
            && mapping.0[3] == "ready".len()
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_TEMPLATE_USE
            && line.trim_start() == crate::virtual_ts::generated_names::TEMPLATE_USE_PREFIX
    }));
}
