<script setup lang="ts">
import { computed } from "vue";

import { navigationRailContext } from "./navigation-rail-context.ts";

const {
  value,
  href = undefined,
  disabled = false,
  badge = undefined,
} = defineProps<{
  /**
   * Destination value; must be one of the root `destinations`.
   *
   * @default required
   */
  readonly value: string;

  /**
   * Link target. With `href` the item renders an `<a>`, otherwise a `<button>`.
   *
   * @default undefined
   */
  readonly href?: string;

  /**
   * Disable the destination.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Badge text rendered with the label (for example an unread count).
   *
   * @default undefined
   */
  readonly badge?: string | number;
}>();

defineSlots<{
  /** Icon and label contents with the active and expanded state. */
  default(props: { readonly active: boolean; readonly expanded: boolean }): unknown;
  /** Visual badge contents; defaults to the `badge` text. */
  badge?(props: { readonly badge: string | number }): unknown;
}>();

const context = navigationRailContext.use();
const active = computed(() => context.active.value === value);
// Script-capable URLs never render; only safe hrefs are bound (spread keeps the attribute absent).
const unsafeHref = /^\s*(?:javascript|vbscript|data):/i;
const linkAttributes = computed(() =>
  href === undefined || disabled || unsafeHref.test(href) ? {} : { href },
);

function onClick(event: MouseEvent): void {
  if (disabled) {
    event.preventDefault();
    return;
  }
  context.select(value, event);
}
</script>

<template>
  <component
    :is="href === undefined ? 'button' : 'a'"
    :type="href === undefined ? 'button' : undefined"
    v-bind="linkAttributes"
    :disabled="href === undefined ? disabled : undefined"
    :aria-disabled="href !== undefined && disabled ? 'true' : undefined"
    :aria-current="active ? 'page' : undefined"
    part="item"
    data-vize-ui="navigation-rail-item"
    :data-value="value"
    :data-state="active ? 'active' : 'inactive'"
    :data-expanded="context.expanded.value ? 'true' : 'false'"
    @click="onClick"
  >
    <slot :active :expanded="context.expanded.value" />
    <span v-if="badge !== undefined" part="badge" data-vize-ui="navigation-rail-badge">
      <slot name="badge" :badge>{{ badge }}</slot>
    </span>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
