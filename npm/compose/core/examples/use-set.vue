<script setup lang="ts">
import { useSet } from "@vizejs/composable/use-set";

const topics = ["Vue", "Vite", "TypeScript", "Testing"];
const { values, has, toggle, clear } = useSet<string>(["Vue"]);
</script>

<template>
  <div class="composable-example">
    <p>Pick the topics you want to follow.</p>
    <div class="example-actions">
      <button
        v-for="topic in topics"
        :key="topic"
        type="button"
        :aria-pressed="has(topic)"
        @click="() => toggle(topic)"
      >
        {{ topic }}
      </button>
      <button type="button" @click="() => clear()">Clear topics</button>
    </div>
    <output aria-live="polite">{{
      values.length === 0 ? "No topics selected." : `Following: ${values.join(", ")}`
    }}</output>
  </div>
</template>
