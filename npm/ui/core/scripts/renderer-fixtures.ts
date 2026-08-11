/** Headless component fixtures compiled by every supported renderer lane. */
export const rendererFixtures = [
  {
    filename: "ScrollLockConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useScrollLock } from "./scroll-lock.ts";

const root = ref<HTMLElement | null>(null);
const ownerDocument = ref<Document | null>(null);
const lock = useScrollLock({ document: ownerDocument, strategy: "auto" });
onMounted(() => {
  ownerDocument.value = root.value?.ownerDocument ?? null;
});
</script>

<template>
  <div ref="root" :data-locked="lock.isLocked.value || undefined">
    Modal content
  </div>
</template>
`,
  },
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
