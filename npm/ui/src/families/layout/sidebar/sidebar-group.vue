<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { sidebarContext, sidebarGroupContext } from "./sidebar-context.ts";
import type { SidebarGroupExpose } from "./sidebar-types.ts";

const { id = undefined } = defineProps<{
  /**
   * Consumer-owned group id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;
}>();

defineSlots<{
  /** SidebarGroupLabel and group contents. */
  default(): unknown;
}>();

const context = sidebarContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const groupId = useDeterministicId({ id: () => id, hint: "sidebar-group" });
const labelId = computed(() => deriveDeterministicId(groupId.value, "label"));

sidebarGroupContext.provide({ labelId });

type SidebarGroupSetupExpose = Omit<SidebarGroupExpose, "element" | "labelId"> & {
  readonly element: typeof element;
  readonly labelId: ComputedRef<string>;
};

const exposed = { element, labelId } satisfies SidebarGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="groupId"
    ref="element"
    role="group"
    :aria-labelledby="labelId"
    data-vize-ui="sidebar-group"
    part="group"
    :data-state="context.state.value"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
