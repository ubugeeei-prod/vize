<script setup lang="ts">
import { computed, ref, useId, useTemplateRef } from "vue";
import { useElementHover } from "@vizejs/composable/element-hover";

const cardsId = useId();
const first = useTemplateRef<HTMLElement>("first");
const second = useTemplateRef<HTMLElement>("second");
const selected = ref("first");
const target = computed(() => (selected.value === "first" ? first.value : second.value));
const hovered = useElementHover(target, { delayEnter: 300, delayLeave: 200 });
const pinned = ref(false);
const guidanceVisible = computed(() => hovered.value || pinned.value);
</script>

<template>
  <div class="composable-example">
    <label :for="cardsId">Guidance target</label>
    <select :id="cardsId" v-model="selected">
      <option value="first">Compiler guide card</option>
      <option value="second">Keyboard guide card</option>
    </select>
    <div class="guide-cards">
      <section ref="first" class="guide-card" aria-label="Compiler guide card">
        <strong>Compiler guide</strong>
        <p>Compile one representative component.</p>
      </section>
      <section ref="second" class="guide-card" aria-label="Keyboard guide card">
        <strong>Keyboard guide</strong>
        <p>Review native input and focus.</p>
      </section>
    </div>
    <button type="button" :aria-pressed="pinned" @click="() => (pinned = !pinned)">
      {{ pinned ? "Hide persistent guidance" : "Show persistent guidance" }}
    </button>
    <output aria-live="polite">
      Delayed pointer hover: {{ hovered ? "yes" : "no" }} · Persistent guidance:
      {{ pinned ? "yes" : "no" }}
    </output>
    <p v-if="guidanceVisible">
      Review the selected card's behavior, diagnostics, and native controls.
    </p>
    <p>
      Pointer guidance waits 300 ms on entry and 200 ms on departure. Switching the selected card
      removes the previous listeners and pending delay. The native guidance button independently
      keeps the same advice available to keyboard and tap users.
    </p>
  </div>
</template>

<style scoped>
.guide-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(220px, 100%), 1fr));
  gap: 12px;
}
.guide-card {
  padding: 12px;
  border: 1px solid currentColor;
  border-radius: 8px;
}
</style>
