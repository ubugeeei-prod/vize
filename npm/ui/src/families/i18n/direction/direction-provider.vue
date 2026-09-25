<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { directionContext, useResolvedDirection } from "./direction-runtime.ts";
import type { Direction, DirectionProviderExpose, DirectionSlotState } from "./direction-types.ts";

const { dir = undefined, as = "div" } = defineProps<{
  /**
   * Direction for the subtree. `undefined` inherits the nearest provider (or `"ltr"`).
   *
   * @default undefined
   */
  readonly dir?: Direction;

  /**
   * Native wrapper element that also receives the `dir` attribute so text and CSS
   * logical properties follow. `null` renders the slot without a wrapper.
   *
   * @default "div"
   */
  readonly as?: keyof HTMLElementTagNameMap | null;
}>();

defineSlots<{
  /** Direction-aware subtree. Receives the resolved direction. */
  default(props: DirectionSlotState): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const resolved = useResolvedDirection(() => dir);
const slotState = computed<DirectionSlotState>(() => ({ dir: resolved.value }));

directionContext.provide(resolved);

type DirectionProviderSetupExpose = Omit<DirectionProviderExpose, "dir" | "element"> & {
  readonly dir: ComputedRef<Direction>;
  readonly element: typeof element;
};

const exposed = { dir: resolved, element } satisfies DirectionProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    v-if="as !== null"
    ref="element"
    :dir="resolved"
    data-vize-ui="direction-provider"
    part="root"
  >
    <slot v-bind="slotState" />
  </component>
  <slot v-else v-bind="slotState" />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
