/** Headless component fixtures compiled by every supported renderer lane. */
export const rendererFixtures = [
  {
    filename: "InertOutsideConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useInertOutside } from "./inert-outside.ts";

const root = ref<HTMLElement | null>(null);
const isolation = useInertOutside({ root, mode: "both" });
</script>

<template>
  <div ref="root" :data-active="isolation.isActive.value || undefined">
    Modal content
  </div>
</template>
`,
  },
  {
    filename: "FocusScopeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useFocusScope } from "./focus-scope.ts";

const root = ref<HTMLElement | null>(null);
const scope = useFocusScope({
  root,
  contain: true,
  autoFocus: true,
  restoreFocus: true,
});
</script>

<template>
  <div ref="root" :data-active="scope.isActive.value || undefined">
    <button type="button">Inside</button>
  </div>
</template>
`,
  },
] as const;
