<script setup lang="ts">
import { ref, computed } from "vue";
import { useInput, TextInput } from "@vizejs/fresco";

interface Todo {
  id: number;
  text: string;
  done: boolean;
}

const todos = ref<Todo[]>([
  { id: 1, text: "Learn Fresco", done: false },
  { id: 2, text: "Build a TUI app", done: false },
  { id: 3, text: "Have fun!", done: true },
]);

const selectedIndex = ref(0);
const inputMode = ref(false);
const newTodoText = ref("");

let nextId = 4;

const stats = computed(() => {
  const total = todos.value.length;
  const done = todos.value.filter((t) => t.done).length;
  return { total, done, remaining: total - done };
});

function toggleTodo() {
  if (todos.value.length === 0) return;
  const todo = todos.value[selectedIndex.value];
  if (todo) {
    todo.done = !todo.done;
  }
}

function deleteTodo() {
  if (todos.value.length === 0) return;
  todos.value.splice(selectedIndex.value, 1);
  if (selectedIndex.value >= todos.value.length) {
    selectedIndex.value = Math.max(0, todos.value.length - 1);
  }
}

function addTodo() {
  if (newTodoText.value.trim()) {
    todos.value.push({
      id: nextId++,
      text: newTodoText.value.trim(),
      done: false,
    });
    newTodoText.value = "";
  }
  inputMode.value = false;
}

function cancelInput() {
  inputMode.value = false;
  newTodoText.value = "";
}

function moveUp() {
  if (selectedIndex.value > 0) {
    selectedIndex.value--;
  }
}

function moveDown() {
  if (selectedIndex.value < todos.value.length - 1) {
    selectedIndex.value++;
  }
}

// Navigation mode only (input mode is handled by TextInput)
const isNavigationMode = computed(() => !inputMode.value);

useInput({
  isActive: isNavigationMode,
  onArrow: (direction) => {
    if (direction === "up") moveUp();
    if (direction === "down") moveDown();
  },
  onChar: (char) => {
    if (char === "j") moveDown();
    if (char === "k") moveUp();
    if (char === " ") toggleTodo();
    if (char === "d") deleteTodo();
    if (char === "a") {
      inputMode.value = true;
    }
  },
});
</script>

<template>
  <box :style="{ flexDirection: 'column', padding: 2, alignItems: 'flex-start' }" border="rounded">
    <text :bold="true" fg="cyan">Todo App</text>
    <text :dim="true">{{ stats.done }}/{{ stats.total }} completed</text>

    <box
      :style="{
        marginTop: 1,
        flexDirection: 'column',
        alignItems: 'flex-start',
      }"
    >
      <box
        v-for="(todo, index) in todos"
        :key="todo.id"
        :style="{ flexDirection: 'row', alignItems: 'flex-start' }"
      >
        <text :fg="index === selectedIndex ? 'yellow' : undefined">{{
          index === selectedIndex ? "❯ " : "  "
        }}</text>
        <text :fg="todo.done ? 'green' : 'white'" :dim="todo.done"
          >{{ todo.done ? "✔" : "○" }} {{ todo.text }}</text
        >
      </box>

      <text v-if="todos.length === 0" :dim="true">No todos yet!</text>
    </box>

    <box v-if="inputMode" :style="{ marginTop: 1, flexDirection: 'row', alignItems: 'flex-start' }">
      <text fg="yellow">> Add: </text>
      <TextInput
        v-model="newTodoText"
        :focus="true"
        fg="yellow"
        @submit="addTodo"
        @cancel="cancelInput"
      />
    </box>

    <box :style="{ marginTop: 1 }">
      <text :dim="true">↑/↓: move, space: toggle, d: delete, a: add, Esc: cancel</text>
    </box>
  </box>
</template>
