<script setup lang="ts">
import { computed } from "vue";

import { SAFE_AREA_EDGES, SAFE_AREA_STYLE, useSafeAreaInsets } from "./safe-area-runtime.ts";
import type { SafeAreaEdge, SafeAreaEdgeInsets } from "./safe-area-runtime.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";

const {
  as = "div",
  edges = SAFE_AREA_EDGES,
  apply = "none",
} = defineProps<{
  /**
   * Element or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Edges this region respects (published as `data-edges` and applied by `apply`).
   *
   * @default ["top", "right", "bottom", "left"]
   */
  readonly edges?: readonly SafeAreaEdge[];

  /**
   * Apply the selected insets as inline padding or margin; `"none"` only exposes CSS variables.
   *
   * @default "none"
   */
  readonly apply?: "margin" | "none" | "padding";
}>();

defineSlots<{
  /** Region contents with the measured insets (zero on the server and before hydration). */
  default?(props: { readonly insets: SafeAreaEdgeInsets }): unknown;
}>();

const { insets } = useSafeAreaInsets();
const style = computed(() => {
  const applied: Record<string, string> = {};
  if (apply !== "none") {
    for (const edge of edges) applied[`${apply}-${edge}`] = `var(--vize-safe-area-inset-${edge})`;
  }
  return { ...SAFE_AREA_STYLE, ...applied };
});
const insetEdges = computed(() => {
  const active = SAFE_AREA_EDGES.filter((edge) => insets.value[edge] > 0);
  return active.length === 0 ? undefined : active.join(" ");
});
</script>

<template>
  <component
    :is="as"
    part="root"
    data-vize-ui="safe-area"
    :data-edges="edges.join(' ')"
    :data-insets="insetEdges"
    :style
  >
    <slot :insets="insets" />
  </component>
</template>

<style scoped>
/* Headless by design. Use var(--vize-safe-area-inset-*) in consumer CSS. */
</style>
