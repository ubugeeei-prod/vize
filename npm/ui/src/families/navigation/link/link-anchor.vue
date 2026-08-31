<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import type { LinkExpose, LinkProps, LinkSlotState } from "./link-types.ts";

const UNSAFE_HREF = /^(?:data|javascript|vbscript):/i;

const {
  id = undefined,
  href = undefined,
  target = undefined,
  rel = undefined,
  download = undefined,
  disabled = false,
  inert = false,
  ariaCurrent = undefined,
} = defineProps<LinkProps>();

defineSlots<{
  /** Renders link contents with current availability state. */
  default(props: LinkSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when an enabled link receives pointer or keyboard activation. */
  navigate: [event: MouseEvent];
}>();

const element = useTemplateRef<HTMLAnchorElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "link" });
const unavailable = computed(() => disabled || inert);
const renderedHref = computed(() => {
  if (unavailable.value || href === undefined) return undefined;
  return normalizeHref(href);
});
const anchorAttributes = computed(() =>
  renderedHref.value === undefined ? {} : { href: renderedHref.value },
);
const renderedTarget = computed(() => (unavailable.value ? undefined : target));
const renderedRel = computed(() => (unavailable.value ? undefined : rel));
const renderedDownload = computed(() => {
  if (unavailable.value || download === false || download === undefined) return undefined;
  return download === true ? "" : download;
});
const ariaCurrentValue = computed(() => {
  if (ariaCurrent === undefined || ariaCurrent === false) return undefined;
  return ariaCurrent === true ? "true" : ariaCurrent;
});
const dataState = computed(() => {
  if (disabled) return "disabled";
  return inert ? "inert" : "idle";
});
const tabIndex = computed(() => (unavailable.value ? -1 : undefined));

function suppressUnavailableActivation(event: Event): void {
  if (!unavailable.value) return;
  event.preventDefault();
  event.stopImmediatePropagation();
}

function onClick(event: MouseEvent): void {
  if (unavailable.value) {
    suppressUnavailableActivation(event);
    return;
  }
  emit("navigate", event);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Enter" || event.key === " ") suppressUnavailableActivation(event);
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

/** Move focus to the native anchor. */
function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type LinkSetupExpose = LinkExpose & { readonly element: typeof element };

const exposed = {
  element,
  focus,
} satisfies LinkSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    is="a"
    :id="controlId"
    ref="element"
    v-bind="anchorAttributes"
    :target="renderedTarget"
    :rel="renderedRel"
    :download="renderedDownload"
    :tabindex="tabIndex"
    :aria-current="ariaCurrentValue"
    :aria-disabled="unavailable ? 'true' : undefined"
    :inert="inert ? true : undefined"
    data-vize-ui="link"
    :data-state="dataState"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot :disabled :inert :unavailable />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
