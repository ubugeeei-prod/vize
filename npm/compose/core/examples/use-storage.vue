<script setup lang="ts">
import { computed, useId } from "vue";
import { useStorage } from "@vizejs/composable/use-storage";

type GuideSection = "getting-started" | "components" | "publishing";

const sectionId = useId();
const {
  state: section,
  supported,
  error,
  remove,
} = useStorage<GuideSection>("vize:docs:reading-progress", "getting-started", {
  writeDefaults: false,
  validate: (candidate): candidate is GuideSection =>
    candidate === "getting-started" || candidate === "components" || candidate === "publishing",
});
const sectionTitle = computed(() => {
  if (section.value === "components") return "Components";
  if (section.value === "publishing") return "Publishing";
  return "Getting started";
});
</script>

<template>
  <div class="composable-example">
    <label :for="sectionId">Guide section</label>
    <select :id="sectionId" v-model="section" :aria-describedby="`${sectionId}-help`">
      <option value="getting-started">Getting started</option>
      <option value="components">Components</option>
      <option value="publishing">Publishing</option>
    </select>
    <p :id="`${sectionId}-help`">
      Choose where to resume reading, then reload this page. Available browser storage keeps your
      choice.
    </p>
    <output aria-live="polite">Resume at: {{ sectionTitle }}</output>
    <p role="status">
      {{
        error
          ? `Storage error: ${error.code}. The current section may not be saved.`
          : supported
            ? "Browser storage is available."
            : "Browser storage is unavailable; changes stay in this tab."
      }}
    </p>
    <button type="button" @click="() => remove()">Reset reading progress</button>
    <p>Reset removes the saved choice and restores Getting started.</p>
  </div>
</template>
