<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useCssVar } from "@vizejs/composable/use-css-var";

const card = useTemplateRef<HTMLElement>("card");
const { value: accent } = useCssVar("--accent", card, { initialValue: "#42b883" });
const colors = [
  { label: "Green", value: "#42b883" },
  { label: "Blue", value: "#3b82f6" },
  { label: "Orange", value: "#f97316" },
];
</script>

<template>
  <div class="composable-example">
    <p>Theme one card by changing a CSS custom property from script.</p>
    <div class="example-actions">
      <button
        v-for="color in colors"
        :key="color.label"
        type="button"
        @click="() => (accent = color.value)"
      >
        {{ color.label }} accent
      </button>
    </div>
    <article ref="card" class="accent-card">
      Accent: <output aria-live="polite">{{ accent }}</output>
    </article>
  </div>
</template>

<style scoped>
.accent-card {
  border: 3px solid var(--accent);
  padding: 12px;
}
</style>
