// The Davinci tab's opening component: small enough to read from the back of
// a room, broad enough that every rung has something to show - a static
// subtree, bindings, an event, v-model on an element and on a component, a
// v-if/v-else chain, a v-for, and a named slot.
export const DAVINCI_PRESET = `<script setup lang="ts">
import { computed, ref } from "vue";
import TodoItem from "./TodoItem.vue";

const draft = ref("");
const todos = ref([{ id: 1, text: "Sketch the disegno", done: true }]);
const remaining = computed(() => todos.value.filter((todo) => !todo.done).length);

function add() {
  todos.value.push({ id: Date.now(), text: draft.value, done: false });
  draft.value = "";
}
</script>

<template>
  <section class="board">
    <h1>Bottega</h1>
    <input v-model="draft" @keyup.enter="add" />
    <ul v-if="todos.length">
      <TodoItem v-for="todo in todos" :key="todo.id" v-model:done="todo.done">
        <template #label>{{ todo.text }}</template>
      </TodoItem>
    </ul>
    <p v-else>Nothing on the easel yet.</p>
    <footer>{{ remaining }} left</footer>
  </section>
</template>
`;

// A mostly static page: the hoist-static analysis marks whole subtrees fully
// static, and S3 partitions almost every op as static.
const STATIC_ISLANDS = `<script setup lang="ts">
const year = new Date().getFullYear();
</script>

<template>
  <article class="poster">
    <header>
      <h1>Vue Fes Japan</h1>
      <p class="tagline">From <em>disegno</em> to fresco</p>
    </header>
    <ul class="talks">
      <li><strong>Surface</strong> the lossless tree</li>
      <li><strong>Disegno</strong> the semantic IR</li>
      <li><strong>Impeto</strong> the reactive graph</li>
    </ul>
    <footer>© {{ year }}</footer>
  </article>
</template>
`;

export interface DavinciExample {
  key: string;
  label: string;
  code: string;
}

/** The tab's examples, in presentation order. */
export const DAVINCI_EXAMPLES: DavinciExample[] = [
  { key: "board", label: "Todo board", code: DAVINCI_PRESET },
  { key: "static", label: "Static islands", code: STATIC_ISLANDS },
];
