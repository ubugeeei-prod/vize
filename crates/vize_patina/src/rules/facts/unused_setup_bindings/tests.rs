use crate::linter::Linter;
use crate::rule::RuleRegistry;
use vize_l0::diag::Tier;

const RULE: &str = "vue/no-unused-setup-bindings";

fn unused(source: &str) -> Vec<(usize, usize)> {
    Linter::with_registry(RuleRegistry::with_opt_in_rules())
        .lint_sfc(source, "Example.vue")
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| (diagnostic.start as usize, diagnostic.end as usize))
        .collect()
}

#[test]
fn actual_opt_in_dispatch_and_exact_identifier_spans() {
    for source in [
        "<script setup>const unused = 0;</script><template><div /></template>",
        "<template><div /></template><script setup>const unused = 0;</script>",
        "<script>export const normal = 0;</script><script setup>const unused = 0;</script><template><div /></template>",
        "<script setup>const unused = 0;</script>",
        "<script setup>const 雪 = '🌸';</script><template><div /></template>",
        "<template lang=\"pug\">\nspan hello\n</template><script setup>const unused = 0;</script>",
    ] {
        let name = if source.contains("const 雪") {
            "雪"
        } else {
            "unused"
        };
        let start = source.find(name).unwrap();
        assert_eq!(unused(source), [(start, start + name.len())], "{source}");
    }
    assert!(!RuleRegistry::with_all().has_rule(RULE));
    assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));
    assert_eq!(
        crate::rule_contracts::contract_for(RULE)
            .unwrap()
            .contract
            .tier(),
        Tier::Sound
    );
}

#[test]
fn script_template_style_component_directive_and_expose_reads_count() {
    for source in [
        "<script setup>const count = 0; const _ignored = 1; console.log(count);</script><template><div /></template>",
        "<script setup>const count = 0; const closure = () => count; defineExpose({closure});</script><template><div /></template>",
        "<script setup>const count = 0;</script><template>{{ count }}</template>",
        "<script setup>const count = 0;</script><template><div :data-count=\"count\" /></template>",
        "<script setup>const color = 'red';</script><template><div /></template><style>div { color: v-bind(color); }</style>",
        "<script setup>const color = {value: 'red'};</script><style>div { color: v-bind('color.value'); }</style>",
        "<script setup>import FooBar from './Foo.vue';</script><template><foo-bar /></template>",
        "<script setup>import * as Foo from './foo';</script><template><Foo.Bar /></template>",
        "<script setup>const vFocus = {};</script><template><div v-focus /></template>",
        "<script setup>const exposed = 0; defineExpose({ public: exposed });</script><template><div /></template>",
        "<script setup lang=\"ts\">const value = {}; type T = typeof value; const _ignored: T = value;</script><template><div /></template>",
        "<script>const normal = 0;</script><template><div /></template>",
        "<script setup>const value = 0;</script><template lang=\"pug\">\nspan {{ value }}\n</template>",
        "<script setup>import { ref } from 'vue'; const el = ref(null);</script><template><div ref=\"el\" /></template>",
        "<script setup>import { ref } from 'vue'; const el = ref(null);</script><template><div :ref=\"'el'\" /></template>",
    ] {
        assert!(unused(source).is_empty(), "{source}");
    }
}

#[test]
fn shadowed_names_and_writes_are_not_reads() {
    for source in [
        "<script setup>const value = 0; const _read = (value) => value;</script><template><div /></template>",
        "<script setup>const value = 0;</script><template><div v-for=\"value in [1]\">{{value}}</div></template>",
        "<script setup>let value; value = 1;</script><template><div /></template>",
        "<script setup>const value = 0;</script><template><input /></template>",
    ] {
        let start = source.find("value").unwrap();
        assert_eq!(unused(source), [(start, start + 5)], "{source}");
    }
}

#[test]
fn unknown_blocks_parse_recovery_and_eval_do_not_prove_non_use() {
    for source in [
        "<script setup>const value = 0; value.;</script><template><div /></template>",
        "<script setup>const value = 0; eval('value');</script><template><div /></template>",
        "<script setup>const value = 0;</script><template src=\"./view.html\" />",
        "<script setup>const value = 0;</script><template><div /></template><style src=\"./style.css\" />",
        "<script setup>const value = 0;</script><template>{{ eval('value') }}</template>",
        "<script setup>const value = 0;</script><template>{{ broken + }}</template>",
        "<script setup>const value = 0;</script><style>div { color: v-bind(broken +); }</style>",
        "<script setup>const value = 0;</script><style>div { color: v-bind(value; }</style>",
        "<script setup>const value = 0;</script><style>div { color: v-bind(eval('value')); }</style>",
        "<script setup lang=\"coffee\">const value = 0;</script><template><div /></template>",
    ] {
        assert!(unused(source).is_empty(), "{source}");
    }
}

#[test]
fn v_for_source_bindings_are_reads() {
    let source = r#"<script setup lang="ts">
import { computed } from "vue";

const DAYS = ["Sun", "Mon", "Tue"];
const rows = computed(() => [[1, 2, 3]]);
const title = "Week";
const unused = 0;
</script>

<template>
  <table>
    <caption>{{ title }}</caption>
    <thead>
      <tr>
        <th v-for="day in DAYS" :key="day">{{ day }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(row, at) in rows" :key="at">
        <td v-for="cell in row" :key="cell">{{ cell }}</td>
      </tr>
    </tbody>
  </table>
</template>
"#;
    let start = source.find("const unused").unwrap() + "const ".len();
    assert_eq!(
        unused(source),
        [(start, start + "unused".len())],
        "{source}"
    );
}

#[test]
fn file_directives_and_configuration_are_honored() {
    let source = "<script setup>\n// eslint-disable-next-line vue/no-unused-setup-bindings\nconst unused = 0;\n</script><template><div /></template>";
    assert!(unused(source).is_empty());
    let source = "<script setup>const unused = 0;</script><template><div /></template>";
    assert!(
        Linter::new()
            .lint_sfc(source, "Example.vue")
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != RULE)
    );
}
