//! Handler functions, `v-for` scope and copied receivers in the template.

use super::{lint_sfc, none};

fn reported_lines(sfc: &str) -> Vec<usize> {
    lint_sfc(sfc)
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
        .map(|diagnostic| {
            sfc.get(..diagnostic.start as usize)
                .map_or(0, |prefix| prefix.matches('\n').count() + 1)
        })
        .collect()
}

#[test]
fn reports_function_handlers_and_the_iterated_prop() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ item: { name: string; count: number }; rows: string[]; names: string[] }>();
</script>

<template>
  <div>
    <button type="button" @click="() => { item.name = 'c'; }">a</button>
    <button type="button" @click="(event) => (item.name = event.type)">b</button>
    <button type="button" @click="() => rows.splice(0, 1)">c</button>
    <button type="button" @click="() => props.item.count++">d</button>
    <ul>
      <li v-for="(row, index) in rows" :key="row" @click="rows.splice(index, 1)">
        <button type="button" @click="rows.sort()">{{ row }}</button>
      </li>
    </ul>
    <button type="button" @click="(rows?.shift)?.()">e</button>
    <button type="button" @click="rows.sort()">f</button>
    <button type="button" @click="item.name = 'c'">g</button>
    <p v-for="name in names" :key="name" @click="rows.splice(0, 1)">{{ name }}</p>
    <button type="button" @click="function () { rows.push('h'); }">h</button>
  </div>
</template>
"#;
    assert_eq!(
        reported_lines(sfc),
        [7, 8, 9, 10, 12, 13, 16, 17, 18, 19, 20]
    );
}

#[test]
fn handler_parameters_and_locals_shadow_props() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ item: { name: string }; rows: string[] }>();
</script>
<template>
  <button @click="(item) => { item.name = 'x'; }">a</button>
  <button @click="() => { const rows = []; rows.push(1); }">b</button>
  <button @click="({ rows }) => rows.push('x')">c</button>
</template>
"#;
    assert_eq!(reported_lines(sfc), Vec::<usize>::new());
}

#[test]
fn ignores_mutating_calls_on_copies_of_a_prop() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ rows: string[] }>();
</script>

<template>
  <div>
    <p>{{ rows.slice().sort().join(", ") }}</p>
    <p :title="rows.filter(Boolean).reverse().join()">a</p>
    <p v-for="row in rows.slice().reverse()" :key="row">{{ row }}</p>
    <p v-if="rows.map((row) => row.trim()).pop()">b</p>
    <button type="button" @click="rows.slice().sort()">c</button>
    <p>{{ rows.length.toString().split("").reverse().join("") }}</p>
    <p>{{ rows.sort().length }}</p>
  </div>
</template>
"#;
    assert_eq!(reported_lines(sfc), [13]);
}

#[test]
fn an_ancestor_v_for_alias_shadows_a_v_model_prop() {
    let sfc = r#"<script setup lang="ts">
interface Item {
  name: string;
}
defineProps<{ items: Item[]; item: Item }>();
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item.name">
      <input v-model="item.name" />
    </li>
  </ul>
</template>
"#;
    assert_eq!(super::findings(&lint_sfc(sfc)), none());
}
