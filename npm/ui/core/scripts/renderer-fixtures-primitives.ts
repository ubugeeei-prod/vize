export const primitiveRendererFixtures = [
  {
    filename: "AspectRatioConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { AspectRatio } from "./aspect-ratio.ts";

const ratio = ref(16 / 9);
</script>

<template>
  <AspectRatio as="figure" :ratio>
    <template #default="{ invalid, ratio: normalizedRatio }">
      <img alt="" src="/poster.png" :data-invalid="invalid || undefined" />
      <figcaption>{{ normalizedRatio }}</figcaption>
    </template>
  </AspectRatio>
</template>
`,
  },
  {
    filename: "SeparatorConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Separator } from "./separator.ts";

const orientation = ref<"horizontal" | "vertical">("vertical");
</script>

<template>
  <Separator as="div" :orientation aria-label="Pane boundary" />
</template>
`,
  },
] as const;
