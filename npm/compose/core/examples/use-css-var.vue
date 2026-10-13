<script setup lang="ts">
import { ref } from "vue";
import { useCssVar } from "@vizejs/composable/use-css-var";

const card = ref<HTMLElement | null>(null);
const { value: accent } = useCssVar("--accent", card, { initialValue: "#42b883" });
const colors = { Green: "#42b883", Blue: "#3b82f6", Orange: "#f97316" };
</script>

<template>
  <div class="composable-example">
    <p>Theme one card by changing a CSS custom property from script.</p>
    <div class="example-actions">
      <button
        v-for="(color, label) in colors"
        :key="label"
        type="button"
        @click="() => (accent = color)"
      >
        {{ label }} accent
      </button>
    </div>
    <article ref="card" style="border: 3px solid var(--accent); padding: 12px">
      Accent: <output aria-live="polite">{{ accent }}</output>
    </article>
  </div>
</template>
