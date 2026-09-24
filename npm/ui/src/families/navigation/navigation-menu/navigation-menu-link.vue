<script setup lang="ts">
import { computed } from "vue";

import { navigationMenuContentContext, navigationMenuContext } from "./navigation-menu-context.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { normalizeNavigationMenuHref } from "./navigation-menu-id.ts";
import type { NavigationMenuLinkSlotState } from "./navigation-menu-types.ts";

const {
  as = "a",
  href = undefined,
  active = false,
} = defineProps<{
  /**
   * Native element, custom element, or router component to render.
   *
   * @default "a"
   */
  readonly as?: PrimitiveAs;

  /**
   * Native link destination. Router components can receive their own route attrs instead.
   *
   * @default undefined
   */
  readonly href?: string;

  /**
   * Whether the link represents the current page (`aria-current="page"`).
   *
   * @default false
   */
  readonly active?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the link is activated. Call `preventDefault()` to keep the flyout open. */
  select: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Link content. Receives whether it represents the current page. */
  default(props: NavigationMenuLinkSlotState): unknown;
}>();

const context = navigationMenuContext.use();
const content = navigationMenuContentContext.useOptional();
const slotState = computed<NavigationMenuLinkSlotState>(() => ({ active }));
// Top-level links join arrow-key navigation between list entries; flyout links do not.
const entry = content === undefined ? "" : undefined;
// Script-capable URLs are dropped so a consumer-supplied href can never execute code.
const anchorAttributes = computed(() => {
  const safe = href === undefined ? undefined : normalizeNavigationMenuHref(href);
  return safe === undefined ? {} : { href: safe };
});

function onClick(event: MouseEvent): void {
  emit("select", event);
  if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.shiftKey) return;
  if (content !== undefined || context.value.value !== null) context.setOpen(null, "link");
}
</script>

<template>
  <component
    :is="as"
    v-bind="anchorAttributes"
    :aria-current="active ? 'page' : undefined"
    :data-navigation-menu-entry="entry"
    data-vize-ui="navigation-menu-link"
    part="link"
    :data-active="active ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
