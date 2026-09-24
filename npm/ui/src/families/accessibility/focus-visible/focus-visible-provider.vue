<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useInteractionModality } from "../interaction-modality/interaction-modality.ts";
import { focusVisibleContext } from "./focus-visible-context.ts";
import { shouldShowFocusRing } from "./focus-visible-runtime.ts";
import type {
  FocusVisibleModality,
  FocusVisibleProviderExpose,
  FocusVisibleSlotState,
} from "./focus-visible-types.ts";

const { attribute = "data-focus-visible", disabled = false } = defineProps<{
  /**
   * Attribute set to `"true"` on the focused descendant while focus should be indicated.
   *
   * @default "data-focus-visible"
   */
  readonly attribute?: string;

  /**
   * Stop publishing modality and focus-visible attributes.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Subtree whose focused descendants receive the focus-visible attribute. */
  default(props: FocusVisibleSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const tracker = useInteractionModality();
const mounted = shallowRef(false);
let focusedElement: Element | null = null;
const marked = shallowRef<Element | null>(null);
const modality = computed<FocusVisibleModality | null>(() =>
  mounted.value && !disabled ? tracker.modality.value : null,
);
const isFocusVisible = computed(() => marked.value !== null);
const slotState = computed<FocusVisibleSlotState>(() => ({
  isFocusVisible: isFocusVisible.value,
  modality: modality.value,
}));

function unmark(): void {
  marked.value?.removeAttribute(attribute);
  marked.value = null;
}

function sync(): void {
  const target = focusedElement;
  if (
    !mounted.value ||
    disabled ||
    target === null ||
    !shouldShowFocusRing(target, tracker.modality.value)
  ) {
    unmark();
    return;
  }
  if (marked.value !== target) unmark();
  target.setAttribute(attribute, "true");
  marked.value = target;
}

function onFocusin(event: FocusEvent): void {
  focusedElement = event.target instanceof Element ? event.target : null;
  sync();
}

function onFocusout(event: FocusEvent): void {
  const next = event.relatedTarget;
  if (next instanceof Node && element.value?.contains(next)) return;
  focusedElement = null;
  sync();
}

watch([() => tracker.modality.value, () => disabled], sync, { flush: "sync" });
watch(
  () => attribute,
  (_next, previous) => {
    marked.value?.removeAttribute(previous);
    marked.value = null;
    sync();
  },
  { flush: "sync" },
);

onMounted(() => {
  mounted.value = true;
  const active = element.value?.ownerDocument.activeElement ?? null;
  if (active && element.value?.contains(active)) focusedElement = active;
  sync();
});

onScopeDispose(unmark);

const focusVisibleModality: Readonly<ShallowRef<FocusVisibleModality | null>> = tracker.modality;

focusVisibleContext.provide({ isFocusVisible, modality: focusVisibleModality });

type FocusVisibleProviderSetupExpose = Omit<
  FocusVisibleProviderExpose,
  "element" | "focusVisibleElement" | "isFocusVisible" | "modality"
> & {
  readonly element: typeof element;
  readonly focusVisibleElement: Readonly<ShallowRef<Element | null>>;
  readonly isFocusVisible: ComputedRef<boolean>;
  readonly modality: ComputedRef<FocusVisibleModality | null>;
};

const exposed = {
  element,
  focusVisibleElement: marked,
  isFocusVisible,
  modality,
} satisfies FocusVisibleProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="focus-visible-provider"
    part="root"
    :data-vize-modality="modality ?? undefined"
    :data-focus-visible-within="isFocusVisible ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    @focusin="onFocusin"
    @focusout="onFocusout"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
