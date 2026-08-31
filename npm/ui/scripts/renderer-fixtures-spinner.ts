export const spinnerRendererFixtures = [
  {
    filename: "SpinnerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Spinner } from "./families/feedback/spinner/spinner.ts";

const value = ref(32);
</script>

<template>
  <Spinner aria-label="Sync progress" aria-value-text="32 of 100" role="progressbar" :value>
    <template #default="{ percent, progressState }">
      <span>{{ progressState }} {{ percent }}</span>
    </template>
  </Spinner>
</template>
`,
  },
] as const;
