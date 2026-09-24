<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { menuGroupContext } from "./menu-context.ts";
import type { MenuElementExpose } from "./menu-types.ts";

const { id = undefined } = defineProps<{
  /**
   * Consumer-owned label id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;
}>();

defineSlots<{
  /** Visible label text. Inside a MenuGroup it names that group. */
  default(): unknown;
}>();

const group = menuGroupContext.useOptional();
const labelId = useDeterministicId({ id: () => id, hint: "menu-label" });
const element = useTemplateRef<HTMLDivElement>("element");

// Wire the group name after mount so server and hydration markup agree.
onMounted(() => {
  if (group) group.labelId.value = labelId.value;
});
onUnmounted(() => {
  if (group?.labelId.value === labelId.value) group.labelId.value = null;
});

defineExpose({ element } satisfies Record<keyof MenuElementExpose, unknown>);
</script>

<template>
  <div :id="labelId" ref="element" role="none" data-vize-ui="menu-label" part="label">
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
