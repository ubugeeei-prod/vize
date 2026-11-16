use super::{ScriptParses, ScriptSemantics, build, lint_scripts};
use crate::{LintPreset, Linter};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

const SOURCE: &str = r#"<template>
  <ul><li v-for="count in items" :key="count">{{ count }}</li></ul>
  <input v-model="props.label" />
  <button @click="$emit('save')">save</button>
</template>
<script lang="ts">
export default { inheritAttrs: false }
</script>
<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
const props = defineProps<{ $el: string; items: string[]; label: string }>()
const count = ref(0)
const items = ref<string[]>([])
const doubled = computed(() => count.value++ * 2)
function add() { props.items.push('x') }
nextTick
</script>
"#;

#[test]
fn each_block_shares_one_pass_and_a_lookup_never_parses() {
    let Ok(descriptor) = parse_sfc(SOURCE, SfcParseOptions::default()) else {
        panic!("fixture parses");
    };
    let allocator = Allocator::default();
    let parses = lint_scripts(&allocator, &descriptor);
    let semantics = ScriptSemantics::new(&parses);
    let foreign_allocator = Allocator::default();
    let foreign = Parser::new(
        &foreign_allocator,
        "export default { inheritAttrs: false }",
        oxc_span::SourceType::ts(),
    )
    .parse();

    // Nothing has read a block yet, so nothing can match and nothing parses.
    assert!(semantics.semantic_of(&foreign.program).is_none());
    assert!(parses.parsed_script().is_none());
    assert!(parses.parsed_script_setup().is_none());

    let Some(setup) = semantics.script_setup() else {
        panic!("setup block");
    };
    let Some(first) = semantics.semantic_of(&setup.program) else {
        panic!("setup pass");
    };
    let Some(again) = semantics.semantic_of(&setup.program) else {
        panic!("setup pass");
    };
    assert!(std::ptr::eq(first, again));
    assert!(parses.parsed_script().is_none());

    let Some(script) = semantics.script() else {
        panic!("script block");
    };
    let Some(sibling) = semantics.semantic_of(&script.program) else {
        panic!("script pass");
    };
    assert!(!std::ptr::eq(first, sibling));
    // Equal text in another parse is not a block parse.
    assert!(semantics.semantic_of(&foreign.program).is_none());
    // A plain holder shares nothing.
    assert!(ScriptParses::semantic_of(&parses, &setup.program).is_none());

    // The shared pass builds nodes; rules that skipped them read the same
    // ids, scopes, symbols, references and diagnostics.
    let without_nodes = build(&setup.program, false);
    let scoping = first.semantic.scoping();
    let other = without_nodes.semantic.scoping();
    assert!(!first.semantic.nodes().is_empty());
    assert!(without_nodes.semantic.nodes().is_empty());
    assert_eq!(first.diagnostics.len(), without_nodes.diagnostics.len());
    assert_eq!(scoping.scopes_len(), other.scopes_len());
    let symbols = |scoping: &oxc_semantic::Scoping| {
        scoping
            .symbol_ids()
            .map(|id| {
                (
                    scoping.symbol_name(id).to_owned(),
                    scoping.symbol_declaration(id),
                    scoping.get_resolved_reference_ids(id).to_vec(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(symbols(scoping), symbols(other));
}

#[test]
fn rules_reading_the_same_blocks_report_as_with_their_own_passes() {
    let result = Linter::with_preset(LintPreset::Opinionated).lint_sfc(SOURCE, "Shared.vue");
    let reports = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.rule_name,
                "vue/no-template-shadow"
                    | "vue/no-mutating-props"
                    | "script/require-explicit-emits"
                    | "script/no-reserved-keys"
                    | "script/no-dupe-keys"
                    | "script/no-side-effects-in-computed-properties"
                    | "script/valid-next-tick"
            )
        })
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.start, diagnostic.end))
        .collect::<Vec<_>>();
    assert_eq!(reports, EXPECTED);
}

/// What these rules reported when each ran its own pass.
const EXPECTED: &[(&str, u32, u32)] = &[
    ("vue/no-template-shadow", 28, 33),
    ("vue/no-mutating-props", 97, 108),
    ("script/require-explicit-emits", 137, 143),
    ("script/no-reserved-keys", 339, 351),
    ("script/no-dupe-keys", 415, 440),
    ("script/no-side-effects-in-computed-properties", 472, 485),
    ("vue/no-mutating-props", 508, 529),
    ("script/valid-next-tick", 532, 540),
];
