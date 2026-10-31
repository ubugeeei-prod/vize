//! Original inline correctness observation, separate from #11210's SSG timing request.
#![expect(
    clippy::disallowed_types,
    reason = "complete authored inputs and output facts"
)]

mod inline_for_ref_shadowing_11210 {
    pub(super) mod support;
}

use inline_for_ref_shadowing_11210::support::compile;
use sha2::{Digest, Sha256};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/inline-for-ref-shadow-11210/Layout.vue.txt"
);
const RENAMED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/inline-for-ref-shadow-11210/renamed.vue.txt"
);
const EXPECTED_INPUT_SHA256: &[u8] = &[
    0xac, 0x78, 0xdd, 0xf1, 0x88, 0xe8, 0x91, 0x63, 0x51, 0x73, 0x83, 0xbc, 0x2d, 0xc4, 0x57, 0xc9,
    0x3d, 0xe7, 0x2b, 0x85, 0xda, 0x49, 0xa6, 0x43, 0x75, 0x8a, 0x78, 0x2d, 0xa3, 0x0b, 0x3f, 0xc8,
];

#[test]
fn original_inline_loop_uses_plain_alias_but_keeps_outer_model_ref() {
    assert_eq!(ORIGINAL.len(), 316);
    let digest = Sha256::digest(ORIGINAL.as_bytes());
    let observed: &[u8] = digest.as_ref();
    assert_eq!(observed, EXPECTED_INPUT_SHA256);
    let facts = compile("original-inline", ORIGINAL, true, false);
    assert_eq!(facts.aliases, ["visibility"]);
    assert_eq!(facts.sources, ["noteVisibilities"]);
    assert_eq!(
        facts.alias_references, 3,
        "key, option value and interpolation"
    );
    assert_eq!(facts.setup_value_references, ["visibility", "visibility"]);
    assert_eq!(facts.assignment_members(), [("visibility", "value")]);
}

#[test]
fn original_non_inline_and_ssr_controls_keep_loop_aliases_plain() {
    for (id, inline, ssr) in [("separate", false, false), ("ssr", false, true)] {
        let facts = compile(id, ORIGINAL, inline, ssr);
        assert_eq!(facts.aliases, ["visibility"]);
        assert!(facts.alias_references >= 3, "{id}: {facts:?}");
        assert!(facts.setup_value_references.is_empty(), "{id}: {facts:?}");
        if !ssr {
            assert_eq!(facts.assignment_members(), [("$setup", "visibility")]);
        }
    }
}

#[test]
fn renaming_only_the_loop_alias_keeps_the_same_outer_model() {
    let renamed = ORIGINAL
        .replace(
            "visibility in noteVisibilities",
            "option in noteVisibilities",
        )
        .replace(":key=\"visibility\"", ":key=\"option\"")
        .replace(":value=\"visibility\"", ":value=\"option\"")
        .replace("{{ visibility }}", "{{ option }}");
    assert_eq!(renamed, RENAMED);
    let facts = compile("renamed-inline", &renamed, true, false);
    assert_eq!(facts.aliases, ["option"]);
    assert_eq!(facts.alias_references, 3);
    assert_eq!(facts.setup_value_references, ["visibility", "visibility"]);
    assert_eq!(facts.assignment_members(), [("visibility", "value")]);
}

#[test]
fn collection_is_evaluated_before_alias_scope_and_siblings_restore_the_ref() {
    let source = r#"<script setup>
import { ref } from 'vue'
const visibility = ref(['public', 'home'])
</script>
<template>
  <select v-model="visibility"><option v-for="visibility in visibility"
    :key="visibility" :value="visibility">{{ visibility }}</option></select>
  <select v-model="visibility" />
  <p>{{ visibility }}</p>
</template>"#;
    let facts = compile("collection-before-alias", source, true, false);
    assert_eq!(facts.sources, ["visibility.value"]);
    assert_eq!(facts.aliases, ["visibility"]);
    assert_eq!(facts.alias_references, 3);
    assert_eq!(
        facts.assignment_members(),
        [("visibility", "value"), ("visibility", "value")]
    );
    assert_eq!(facts.setup_value_references.len(), 6, "{facts:?}");
}

#[test]
fn nested_aliases_keys_compound_reads_and_handler_locals_keep_lexical_precedence() {
    let source = r#"<script setup>
import { ref } from 'vue'
const visibility = ref('public')
const index = ref(10)
const rows = ref([['public', 'home']])
</script>
<template>
  <select v-model="visibility" />
  <div v-for="(visibility, index) in rows" :key="index">
    <span :data-pair="[visibility, index]">{{ `${index}:${visibility}` }}</span>
    <button @click="(visibility) => visibility + index">{{ visibility }}</button>
    <b v-for="(visibility, index) in visibility" :key="index"
      :data-pair="[visibility, index]">{{ visibility }}</b>
    <i>{{ visibility }}{{ index }}</i>
  </div>
  <select v-model="visibility" />
  <p>{{ visibility }}{{ index }}</p>
</template>"#;
    let facts = compile("nested-and-handler", source, true, false);
    assert_eq!(facts.sources, ["rows.value", "visibility"]);
    assert_eq!(
        facts.aliases,
        ["visibility", "index", "visibility", "index"]
    );
    assert!(facts.alias_references >= 14, "{facts:?}");
    assert_eq!(
        facts.assignment_members(),
        [("visibility", "value"), ("visibility", "value")]
    );
    assert_eq!(
        facts
            .setup_value_references
            .iter()
            .filter(|name| *name == "visibility")
            .count(),
        5
    );
    assert_eq!(
        facts
            .setup_value_references
            .iter()
            .filter(|name| *name == "index")
            .count(),
        1
    );
}
