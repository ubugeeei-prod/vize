<script setup lang="ts">
import { shallowRef, useTemplateRef } from "vue";

import { menuGroupContext } from "./menu-context.ts";
import type { MenuElementExpose } from "./menu-types.ts";

defineSlots<{
  /** Grouped items, usually led by a MenuLabel that names the group. */
  default(): unknown;
}>();

const labelId = shallowRef<string | null>(null);
const element = useTemplateRef<HTMLDivElement>("element");

menuGroupContext.provide({ labelId });

defineExpose({ element } satisfies Record<keyof MenuElementExpose, unknown>);
</script>

<template>
  <div
    ref="element"
    role="group"
    :aria-labelledby="labelId ?? undefined"
    data-vize-ui="menu-group"
    part="group"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
