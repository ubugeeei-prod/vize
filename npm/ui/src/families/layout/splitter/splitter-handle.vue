<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { splitterContext } from "./splitter-context.ts";
import type {
  SplitterHandleExpose,
  SplitterHandleSlotState,
  SplitterHandleState,
  SplitterOrientation,
} from "./splitter-types.ts";

const {
  id = undefined,
  disabled = false,
  order = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Consumer-owned separator id.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Disable this handle while keeping the rest of the group resizable.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Deterministic order for conditionally rendered handles.
   *
   * @default undefined
   */
  readonly order?: number;

  /**
   * Accessible name, for example "Resize sidebar".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the separator.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Optional grip content. Receives the value, orientation, and drag state. */
  default?(props: SplitterHandleSlotState): unknown;
}>();

const context = splitterContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const handleId = useDeterministicId({ id: () => id, hint: "splitter-handle" });
let registration: CollectionRegistration<string> | null = null;

watch(
  handleId,
  (next) => {
    registration?.unregister();
    registration = context.registerHandle({ element, id: next, order: () => order });
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  registration?.unregister();
  registration = null;
});

const handleIndex = computed(() => context.getHandleIndex(handleId.value));
const constraints = computed(() => context.getPanelConstraints(handleIndex.value));
const controlsId = computed(() => context.getPanelId(handleIndex.value));
const value = computed(() => Math.round(context.getSize(handleIndex.value)));
const valueMin = computed(() =>
  Math.round(
    constraints.value?.collapsible === true
      ? constraints.value.collapsedSize
      : (constraints.value?.minSize ?? 0),
  ),
);
const valueMax = computed(() => Math.round(constraints.value?.maxSize ?? 100));
const handleDisabled = computed(() => disabled || context.disabled.value);
const dragging = computed(() => context.draggingHandle.value === handleId.value);
// The separator sits between panels, so it is perpendicular to the group axis.
const separatorOrientation = computed<SplitterOrientation>(() =>
  context.orientation.value === "horizontal" ? "vertical" : "horizontal",
);
const handleState = computed<SplitterHandleState>(() => {
  if (handleDisabled.value) return "disabled";
  return dragging.value ? "dragging" : "idle";
});
const slotState = computed<SplitterHandleSlotState>(() => ({
  disabled: handleDisabled.value,
  dragging: dragging.value,
  orientation: context.orientation.value,
  state: handleState.value,
  value: value.value,
}));

function onPointerdown(event: PointerEvent): void {
  if (handleDisabled.value) return;
  element.value?.focus({ preventScroll: true });
  context.startDrag(handleId.value, event);
}

function currentIndex(): number {
  return handleIndex.value;
}

function onKeydown(event: KeyboardEvent): void {
  if (handleDisabled.value || event.defaultPrevented) return;
  const index = currentIndex();
  const constraint = context.getPanelConstraints(index);
  if (constraint === undefined) return;
  const step = context.keyboardStep.value;
  const horizontal = context.orientation.value === "horizontal";
  const rtl = context.dir.value === "rtl";
  const size = context.getSize(index);
  let handled = true;
  switch (event.key) {
    case "ArrowLeft":
      if (horizontal) context.resizeHandle(index, rtl ? step : -step, "keyboard");
      else handled = false;
      break;
    case "ArrowRight":
      if (horizontal) context.resizeHandle(index, rtl ? -step : step, "keyboard");
      else handled = false;
      break;
    case "ArrowUp":
      if (horizontal) handled = false;
      else context.resizeHandle(index, -step, "keyboard");
      break;
    case "ArrowDown":
      if (horizontal) handled = false;
      else context.resizeHandle(index, step, "keyboard");
      break;
    case "Home": {
      const floor = constraint.collapsible ? constraint.collapsedSize : constraint.minSize;
      context.resizeHandle(index, floor - size, "keyboard");
      break;
    }
    case "End":
      context.resizeHandle(index, constraint.maxSize - size, "keyboard");
      break;
    case "Enter":
      if (!constraint.collapsible) handled = false;
      else if (size <= constraint.collapsedSize) context.expandPanel(index, "keyboard");
      else context.collapsePanel(index, "keyboard");
      break;
    default:
      handled = false;
  }
  if (handled) event.preventDefault();
}

const separatorProps = computed<{
  readonly role: "separator";
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
}>(() => ({ role: "separator", onKeydown, onPointerdown }));

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type SplitterHandleSetupExpose = Omit<SplitterHandleExpose, "element" | "value"> & {
  readonly element: typeof element;
  readonly value: ComputedRef<number>;
};

const exposed = {
  element,
  focus,
  value,
} satisfies SplitterHandleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    v-bind="separatorProps"
    :id="handleId"
    ref="element"
    :tabindex="handleDisabled ? undefined : 0"
    :aria-orientation="separatorOrientation"
    :aria-valuenow="value"
    :aria-valuemin="valueMin"
    :aria-valuemax="valueMax"
    :aria-controls="controlsId"
    :aria-disabled="handleDisabled ? 'true' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    data-vize-ui="splitter-handle"
    part="handle"
    :data-state="handleState"
    :data-orientation="context.orientation.value"
    :data-disabled="handleDisabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
