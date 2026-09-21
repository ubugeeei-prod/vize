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
