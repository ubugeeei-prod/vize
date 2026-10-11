<script setup lang="ts">
import { useId } from "vue";
import { useColorMode } from "@vizejs/composable/color-mode";

const choiceId = useId();
const { mode, system, state, stored } = useColorMode<"contrast">({
  attribute: "data-guide-mode",
  storageKey: "vize:docs:color-mode",
  modes: { contrast: "contrast" },
  disableTransition: false,
});
</script>

<template>
  <div class="composable-example">
    <label :for="choiceId">Guide color preference</label>
    <select :id="choiceId" v-model="mode">
      <option value="auto">Automatic</option>
      <option value="dark">Dark</option>
      <option value="light">Light</option>
      <option value="contrast">Contrast</option>
    </select>
    <section class="mode-card" aria-label="Color preference sample">
      <h3>Read the guide in your chosen mode</h3>
      <p>The automatic choice follows this browser's actual system preference.</p>
    </section>
    <output aria-live="polite"
      >Chosen: {{ mode }} · System: {{ system }} · Applied: {{ state }}</output
    >
    <p>Preference read at mount: {{ stored }}.</p>
    <button type="button" @click="() => (mode = 'auto')">Follow system preference</button>
    <p>
      A manual choice overrides later system changes. Contrast is an explicitly configured custom
      mode. This example applies an attribute to this preview's own HTML element; it does not change
      the parent documentation. The in-tab choice still applies if browser storage cannot persist
      it. Server rendering starts with Automatic and a Light system fallback, then the client reads
      the actual preference after mounting.
    </p>
  </div>
</template>

<style scoped>
.mode-card {
  border: 1px solid currentColor;
  border-radius: 8px;
  padding: 12px;
  background: #ffffff;
  color: #182833;

  & h3 {
    margin-top: 0;
  }
}
:global(html[data-guide-mode="dark"]) {
  & .mode-card {
    background: #182833;
    color: #fff9ee;
  }
}
:global(html[data-guide-mode="contrast"]) {
  & .mode-card {
    background: #000000;
    color: #fff37c;
  }
}
</style>
