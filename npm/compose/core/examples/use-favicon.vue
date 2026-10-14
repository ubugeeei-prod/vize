<script setup lang="ts">
import { computed, ref } from "vue";
import { useFavicon } from "@vizejs/composable/use-favicon";

const status = ref<"passing" | "failing">("passing");
const dot = (color: string) =>
  `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><circle cx="8" cy="8" r="7" fill="${color}"/></svg>`)}`;
useFavicon(computed(() => dot(status.value === "passing" ? "#16a34a" : "#dc2626")));
</script>

<template>
  <div class="composable-example">
    <p>Change the tab icon to reflect a build status.</p>
    <div class="example-actions">
      <button type="button" @click="() => (status = 'passing')">Build passed</button>
      <button type="button" @click="() => (status = 'failing')">Build failed</button>
    </div>
    <output aria-live="polite">Favicon shows the {{ status }} build.</output>
  </div>
</template>
