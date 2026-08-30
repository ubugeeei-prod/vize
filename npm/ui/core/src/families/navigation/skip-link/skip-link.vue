<script setup lang="ts">
import { computed, ref, useTemplateRef } from "vue";
import type { ComputedRef, Ref } from "vue";

import { useDeterministicId } from "../../../deterministic-id.ts";
import type {
  SkipLinkActivation,
  SkipLinkExpose,
  SkipLinkFocusResult,
  SkipLinkHref,
  SkipLinkProps,
  SkipLinkSlotState,
  SkipLinkState,
} from "./skip-link-types.ts";

const DEFAULT_HREF = "#main" satisfies SkipLinkHref;

const {
  id = undefined,
  href = DEFAULT_HREF,
  focusTarget: shouldFocusTarget = true,
} = defineProps<SkipLinkProps>();

defineSlots<{
  /** Renders skip-link contents with the resolved hash target and focus state. */
  default(props: SkipLinkSlotState): unknown;
}>();

const emit = defineEmits<{
  /** Fired when a valid skip link receives native pointer or keyboard activation. */
  activate: [event: MouseEvent, detail: SkipLinkActivation];
}>();

const element = useTemplateRef<HTMLAnchorElement>("element");
const linkId = useDeterministicId({ id: () => id, hint: "skip-link" });
const focused = ref(false);
const renderedHref = computed<SkipLinkHref | undefined>(() => normalizeHashHref(href));
const anchorAttributes = computed(() =>
  renderedHref.value === undefined ? {} : { href: renderedHref.value },
);
const targetId = computed<string | undefined>(() => renderedHref.value?.slice(1));
const unavailable = computed<boolean>(() => renderedHref.value === undefined);
const state = computed<SkipLinkState>(() => {
  if (unavailable.value) return "invalid";
  return focused.value ? "focused" : "idle";
});
const slotState = computed<SkipLinkSlotState>(() => ({
  focused: focused.value,
  href: renderedHref.value,
  state: state.value,
  targetId: targetId.value,
  unavailable: unavailable.value,
}));

function onFocus(): void {
  focused.value = true;
}

function onBlur(): void {
  focused.value = false;
}

function onClick(event: MouseEvent): void {
  const current = getCurrentLink();
  if (current === null) {
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }

  const result = shouldFocusTarget
    ? focusTarget()
    : ({ target: getTarget(), focused: false } satisfies SkipLinkFocusResult);

  emit("activate", event, {
    focused: result.focused,
    href: current.href,
    target: result.target,
    targetId: current.targetId,
  });
}

function getCurrentLink(): { readonly href: SkipLinkHref; readonly targetId: string } | null {
  if (renderedHref.value === undefined || targetId.value === undefined) return null;
  return { href: renderedHref.value, targetId: targetId.value };
}

function normalizeHashHref(value: SkipLinkHref): SkipLinkHref | undefined {
  const normalized = value.trim();
  if (!normalized.startsWith("#") || normalized.length === 1) return undefined;
  return hasAsciiWhitespaceOrControl(normalized) ? undefined : (normalized as SkipLinkHref);
}

function hasAsciiWhitespaceOrControl(value: string): boolean {
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x20 || code === 0x7f) return true;
  }
  return false;
}

function getTarget(): HTMLElement | null {
  if (targetId.value === undefined || typeof document === "undefined") return null;
  const target = document.getElementById(targetId.value);
  return target instanceof HTMLElement ? target : null;
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

function focusTarget(options?: FocusOptions): SkipLinkFocusResult {
  const target = getTarget();
  if (target === null) return { target: null, focused: false };

  const hadTabIndex = target.hasAttribute("tabindex");
  if (!hadTabIndex) target.setAttribute("tabindex", "-1");
  target.focus(options);

  const movedFocus = target.ownerDocument.activeElement === target;
  if (!hadTabIndex) {
    const restoreTabIndex = (): void => target.removeAttribute("tabindex");
    if (movedFocus) target.addEventListener("blur", restoreTabIndex, { once: true });
    else restoreTabIndex();
  }

  return { target, focused: movedFocus };
}

type SkipLinkSetupExpose = Omit<
  SkipLinkExpose,
  "element" | "focused" | "href" | "state" | "targetId" | "unavailable"
> & {
  readonly element: typeof element;
  readonly focused: Ref<boolean>;
  readonly href: ComputedRef<SkipLinkHref | undefined>;
  readonly state: ComputedRef<SkipLinkState>;
  readonly targetId: ComputedRef<string | undefined>;
  readonly unavailable: ComputedRef<boolean>;
};

const exposed = {
  element,
  focus,
  focused,
  focusTarget,
  getTarget,
  href: renderedHref,
  state,
  targetId,
  unavailable,
} satisfies SkipLinkSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    is="a"
    :id="linkId"
    ref="element"
    v-bind="anchorAttributes"
    :aria-disabled="unavailable ? 'true' : undefined"
    :tabindex="unavailable ? -1 : undefined"
    data-vize-ui="skip-link"
    part="root"
    :data-state="state"
    :data-target-id="targetId"
    :data-unavailable="unavailable ? 'true' : undefined"
    @click="onClick"
    @focus="onFocus"
    @blur="onBlur"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Visibility, placement, focus ring, and motion remain consumer-owned. */
</style>
