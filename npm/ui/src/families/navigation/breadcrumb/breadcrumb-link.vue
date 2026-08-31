<template>
  <component
    :is="as"
    ref="element"
    v-bind="anchorAttributes"
    :aria-current="ariaCurrentState"
    data-vize-ui="breadcrumb-link"
    part="link"
    :data-current="currentState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  BreadcrumbCurrent,
  BreadcrumbLinkExpose,
  BreadcrumbLinkSlotState,
} from "./breadcrumb-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

const UNSAFE_HREF = /^(?:data|javascript|vbscript):/i;

const {
  as = "a",
  current = false,
  href = undefined,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "a"
   */
  readonly as?: PrimitiveAs;

  /**
   * Current route state. `true` resolves to `aria-current="page"`.
   *
   * @default false
   */
  readonly current?: BreadcrumbCurrent | false;

  /**
   * Native link destination. Router components can receive their own route attrs.
   *
   * @default undefined
   */
  readonly href?: string;
}>();

defineSlots<{
  /** Renders link content with resolved current-route state. */
  default(props: BreadcrumbLinkSlotState): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");
const currentState = computed(() => current !== false);
const renderedHref = computed(() => {
  if (href === undefined) return undefined;
  return normalizeHref(href);
});
const anchorAttributes = computed(() =>
  renderedHref.value === undefined ? {} : { href: renderedHref.value },
);
const ariaCurrentState = computed<BreadcrumbLinkSlotState["ariaCurrent"]>(() => {
  if (current === false) return undefined;
  return current === true ? "page" : current;
});
const slotState = computed<BreadcrumbLinkSlotState>(() => ({
  ariaCurrent: ariaCurrentState.value,
  current: currentState.value,
}));

function focus(options?: FocusOptions): void {
  if (element.value instanceof HTMLElement) element.value.focus(options);
}

function normalizeHref(value: string): string | undefined {
  const normalized = value.trim();
  if (normalized.length === 0 || hasControlCharacter(normalized) || UNSAFE_HREF.test(normalized))
    return undefined;
  return normalized;
}

function hasControlCharacter(value: string): boolean {
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code < 32 || code === 127) return true;
  }
  return false;
}

type BreadcrumbLinkSetupExpose = Omit<
  BreadcrumbLinkExpose,
  "ariaCurrent" | "current" | "element"
> & {
  readonly ariaCurrent: typeof ariaCurrentState;
  readonly current: typeof currentState;
  readonly element: typeof element;
};

const exposed = {
  ariaCurrent: ariaCurrentState,
  current: currentState,
  element,
  focus,
} satisfies BreadcrumbLinkSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. Link color, underline, hover, and current-route styling remain consumer-owned. */
</style>
