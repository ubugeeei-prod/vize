<script setup lang="ts">
import { computed, ref } from "vue";
import { useWindowSize } from "@vizejs/composable/window-size";

const { width, height, isSupported } = useWindowSize();
const layout = ref<"automatic" | "single">("automatic");
const columns = computed(() => (layout.value === "automatic" && width.value >= 640 ? 2 : 1));
const showMeasurements = ref(false);
const draft = ref("");
</script>

<template>
  <div class="composable-example">
    <label>
      Guide layout
      <select v-model="layout">
        <option value="automatic">Automatic</option>
        <option value="single">Single column</option>
      </select>
    </label>
    <div
      class="viewport-guides"
      :style="{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }"
    >
      <section aria-label="Compiler review card">
        <strong>Compiler</strong>
        <p>Inspect the generated render function.</p>
      </section>
      <section aria-label="Type check review card">
        <strong>Type checking</strong>
        <p>Review diagnostics against the same source.</p>
      </section>
    </div>
    <label class="measurement-choice">
      <input v-model="showMeasurements" type="checkbox" /> Show layout measurements
    </label>
    <p v-if="showMeasurements">
      Layout: {{ columns }} {{ columns === 1 ? "column" : "columns" }} · Choice: {{ layout }}
    </p>
    <label>
      Viewport review draft
      <input v-model="draft" type="text" placeholder="Local draft survives a resize" />
    </label>
    <output aria-live="polite">
      Viewport: {{ width }} × {{ height }} CSS pixels · Window support:
      {{ isSupported ? "available" : "unavailable" }}
    </output>
    <p>
      Resize this preview's viewport to update its readonly dimensions. Automatic uses two columns
      from 640 px; Single column remains your choice at any width. Resizing preserves the local
      draft and measurement visibility.
    </p>
    <p>
      Default inner dimensions include scrollbars and describe this iframe, not the parent page or
      the outer browser window. SSR starts at 0 × 0 with support unavailable; after mounting, the
      real window supplies its dimensions.
    </p>
  </div>
</template>

<style scoped>
.viewport-guides {
  display: grid;
  gap: 12px;

  & section {
    min-inline-size: 0;
    padding: 12px;
    border: 1px solid currentColor;
    border-radius: 8px;
  }
}
.measurement-choice {
  display: inline-flex;
  align-items: center;
  gap: 6px;

  & input {
    inline-size: auto;
  }
}
</style>
