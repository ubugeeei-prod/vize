<script setup lang="ts">
import { navigationMenuContext } from "./navigation-menu-context.ts";
import type { NavigationMenuOrientation } from "./navigation-menu-types.ts";

defineSlots<{
  /** NavigationMenuItem children. Receives the list axis. */
  default(props: {
    /** Layout axis of the list. */
    readonly orientation: NavigationMenuOrientation;
  }): unknown;
}>();

const context = navigationMenuContext.use();
</script>

<template>
  <ul data-vize-ui="navigation-menu-list" part="list" :data-orientation="context.orientation.value">
    <slot :orientation="context.orientation.value" />
  </ul>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
