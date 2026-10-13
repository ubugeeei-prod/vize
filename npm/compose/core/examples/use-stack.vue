<script setup lang="ts">
import { computed } from "vue";
import { useStack } from "@vizejs/composable/use-stack";

const { items, size, push, pop } = useStack<string>(["Home"]);
const current = computed(() => items.value.at(-1) ?? "Home");
const pages = ["Docs", "Guide", "API"];
</script>

<template>
  <div class="composable-example">
    <p>Keep a back stack for in-app navigation.</p>
    <nav aria-label="Pages" class="example-actions">
      <button v-for="page in pages" :key="page" type="button" @click="() => push(page)">
        Open {{ page }}
      </button>
      <button type="button" :disabled="size <= 1" @click="() => pop()">Back</button>
    </nav>
    <output aria-live="polite"
      >Current page: {{ current }} · History: {{ items.join(" › ") }}</output
    >
  </div>
</template>
