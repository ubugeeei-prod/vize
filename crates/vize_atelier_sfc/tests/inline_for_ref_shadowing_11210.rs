//! Original inline correctness observation, separate from #11210's SSG timing request.
#![expect(
    clippy::disallowed_types,
    reason = "complete authored inputs and output facts"
)]

#[path = "inline_for_ref_shadowing_11210/support.rs"]
mod support;

use sha2::{Digest, Sha256};
use support::compile;

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/inline-for-ref-shadow-11210/Layout.vue.txt"
);

#[test]
fn original_inline_loop_uses_plain_alias_but_keeps_outer_model_ref() {
    assert_eq!(ORIGINAL.len(), 316);
    assert_eq!(
        vize_l0::cstr!("{:x}", Sha256::digest(ORIGINAL.as_bytes())).as_str(),
        "ac78ddf188e89163517383bc2dc457c93de72b85da49a643758a782da30b3fc8"
    );
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
    assert!(renamed.contains("v-model=\"visibility\""));
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
