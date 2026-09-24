<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { MenuElementExpose } from "./menu-types.ts";

const element = useTemplateRef<HTMLDivElement>("element");
// A static, non-focusable separator: no value range applies.
const separatorProps = Object.freeze({
  role: "separator",
  "aria-orientation": "horizontal",
} as const);

defineExpose({ element } satisfies Record<keyof MenuElementExpose, unknown>);
</script>

<template>
  <div v-bind="separatorProps" ref="element" data-vize-ui="menu-separator" part="separator"></div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
