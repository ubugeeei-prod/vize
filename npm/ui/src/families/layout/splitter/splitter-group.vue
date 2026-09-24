<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { splitterContext } from "./splitter-context.ts";
import type {
  SplitterContextValue,
  SplitterHandleRegistrationInput,
  SplitterPanelRegistrationInput,
} from "./splitter-context.ts";
import {
  clampSplitterLayout,
  isValidSplitterLayout,
  resizeSplitterLayout,
  resolveDefaultSplitterLayout,
  splitterLayoutsEqual,
} from "./splitter-layout.ts";
import type {
  SplitterDirection,
  SplitterGroupExpose,
  SplitterGroupSlotState,
  SplitterGroupState,
  SplitterLayout,
  SplitterOrientation,
  SplitterPanelConstraints,
  SplitterResizeReason,
} from "./splitter-types.ts";

const {
  id = undefined,
  layout = undefined,
  defaultLayout = undefined,
  orientation = "horizontal",
  dir = "ltr",
  disabled = false,
  keyboardStep = 10,
} = defineProps<{
  /**
   * Consumer-owned group id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled panel sizes in percent (`v-model:layout`). `undefined` selects uncontrolled
   * behavior, which also lets `useSplitterPersistence` load a stored layout after mount.
   *
   * @default undefined
   */
  readonly layout?: SplitterLayout;

  /**
   * Initial sizes by panel index. Panels without an entry use their `defaultSize`.
   *
   * @default undefined
   */
  readonly defaultLayout?: SplitterLayout;

  /**
   * Layout axis. `"horizontal"` places panels side by side with vertical separators.
   *
   * @default "horizontal"
   */
  readonly orientation?: SplitterOrientation;

  /**
   * Reading direction used to map horizontal arrow keys and pointer movement.
   *
   * @default "ltr"
   */
  readonly dir?: SplitterDirection;

  /**
   * Disable pointer and keyboard resizing while keeping the current layout.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Percent moved by one arrow key press on a handle.
   *
   * @default 10
   */
  readonly keyboardStep?: number;
}>();

const emit = defineEmits<{
  /** Fired with the next layout whenever sizes change. */
  "update:layout": [layout: SplitterLayout];

  /** Fired after a distinct layout change with the input that caused it. */
  resize: [layout: SplitterLayout, reason: SplitterResizeReason];

  /** Fired when a pointer drag starts on a handle. */
  resizeStart: [handleIndex: number];

  /** Fired when a pointer drag ends. */
  resizeEnd: [layout: SplitterLayout];
}>();

defineSlots<{
  /** SplitterPanel and SplitterHandle children. Receives the resolved layout and drag state. */
  default(props: SplitterGroupSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "splitter" });
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const disabledState = computed(() => disabled);
const keyboardStepState = computed(() => keyboardStep);
const panels = createCollectionRegistry<string, SplitterPanelRegistrationInput>();
const handles = createCollectionRegistry<string, SplitterHandleRegistrationInput>();
const draggingHandle = shallowRef<string | null>(null);
const expandedSizes = new Map<string, number>();

const constraints = computed<readonly SplitterPanelConstraints[]>(() =>
  panels.items.value.map((item) => item.value.constraints()),
);
// Defaults are resolved per panel without rebalancing, so a panel rendered before its
// siblings register (server rendering) already has its final size.
const defaults = computed(() =>
  resolveDefaultSplitterLayout(
    panels.items.value.map((item, index) => defaultLayout?.[index] ?? item.value.defaultSize()),
  ).map((size, index) => {
    const constraint = constraints.value[index];
    return constraint === undefined
      ? size
      : Math.min(constraint.maxSize, Math.max(constraint.minSize, size));
  }),
);
const layoutState = useControllableState<SplitterLayout | undefined>({
  value: () => layout,
  defaultValue: () => undefined,
  equals: splitterLayoutsEqual,
});
// Panels register one by one while the group renders, so before mount (and on the
// server) a longer valid layout is read by index; after mount the counts must match.
const mounted = shallowRef(false);
const resolvedLayout = computed<SplitterLayout>(() => {
  const current = layoutState.value.value;
  const count = panels.items.value.length;
  if (current !== undefined && isValidSplitterLayout(current, count)) return current;
  if (!mounted.value && current !== undefined && current.length > count) {
    if (isValidSplitterLayout(current, current.length)) return current.slice(0, count);
  }
  return defaults.value;
});
const state = computed<SplitterGroupState>(() => {
  if (disabledState.value) return "disabled";
  return draggingHandle.value === null ? "idle" : "resizing";
});
const slotState = computed<SplitterGroupSlotState>(() => ({
  disabled: disabledState.value,
  layout: resolvedLayout.value,
  orientation: orientationState.value,
  resizing: draggingHandle.value !== null,
  state: state.value,
}));
const groupStyle = computed(() => ({
  display: "flex",
  flexDirection: orientationState.value === "horizontal" ? ("row" as const) : ("column" as const),
}));

function currentLayout(): SplitterLayout {
  return resolvedLayout.value;
}

function commit(next: SplitterLayout, reason: SplitterResizeReason): boolean {
  if (splitterLayoutsEqual(next, currentLayout())) return false;
  const frozen = Object.freeze([...next]);
  layoutState.set(frozen);
  emit("update:layout", frozen);
  emit("resize", frozen, reason);
  return true;
}

function setLayout(next: SplitterLayout): boolean {
  if (!isValidSplitterLayout(next, panels.items.value.length)) return false;
  return commit(clampSplitterLayout(next, constraints.value), "programmatic");
}

function reset(): boolean {
  const target =
    defaultLayout !== undefined && isValidSplitterLayout(defaultLayout, panels.items.value.length)
      ? clampSplitterLayout(defaultLayout, constraints.value)
      : defaults.value;
  return commit(target, "programmatic");
}

function getSize(index: number): number {
  return resolvedLayout.value[index] ?? 0;
}

function resizeHandle(index: number, delta: number, reason: SplitterResizeReason): boolean {
  if (disabledState.value) return false;
  return commit(resizeSplitterLayout(currentLayout(), constraints.value, index, delta), reason);
}

function setPanelSize(index: number, size: number, reason: SplitterResizeReason): boolean {
  const count = panels.items.value.length;
  const delta = size - getSize(index);
  if (index < count - 1) return resizeHandle(index, delta, reason);
  return resizeHandle(index - 1, -delta, reason);
}

function collapsePanel(index: number, reason: SplitterResizeReason): boolean {
  const constraint = constraints.value[index];
  const panelId = panels.items.value[index]?.key;
  const size = getSize(index);
  if (constraint === undefined || panelId === undefined || !constraint.collapsible) return false;
  if (size <= constraint.collapsedSize) return false;
  const changed = setPanelSize(index, constraint.collapsedSize, reason);
  if (changed) expandedSizes.set(panelId, size);
  return changed;
}

function expandPanel(index: number, reason: SplitterResizeReason): boolean {
  const constraint = constraints.value[index];
  const panelId = panels.items.value[index]?.key;
  if (constraint === undefined || panelId === undefined) return false;
  if (getSize(index) >= constraint.minSize) return false;
  return setPanelSize(index, expandedSizes.get(panelId) ?? constraint.minSize, reason);
}

let stopDrag: (() => void) | null = null;

function groupElement(): HTMLDivElement | null {
  return element.value;
}

function startDrag(handleId: string, event: PointerEvent): void {
  const handleIndex = handles.items.value.findIndex((item) => item.key === handleId);
  const group = groupElement();
  if (disabledState.value || handleIndex < 0 || group === null || event.button !== 0) return;
  stopDrag?.();
  const horizontal = orientationState.value === "horizontal";
  const rect = group.getBoundingClientRect();
  const extent = horizontal ? rect.width : rect.height;
  const origin = horizontal ? event.clientX : event.clientY;
  const startLayout = currentLayout();
  const view = group.ownerDocument.defaultView;
  if (view === null) return;
  event.preventDefault();
  draggingHandle.value = handleId;
  emit("resizeStart", handleIndex);

  const onMove = (move: PointerEvent) => {
    if (extent <= 0) return;
    const position = horizontal ? move.clientX : move.clientY;
    let delta = ((position - origin) / extent) * 100;
    if (horizontal && dirState.value === "rtl") delta = -delta;
    commit(resizeSplitterLayout(startLayout, constraints.value, handleIndex, delta), "pointer");
  };
  const stop = () => {
    view.removeEventListener("pointermove", onMove);
    view.removeEventListener("pointerup", stop);
    view.removeEventListener("pointercancel", stop);
    stopDrag = null;
    draggingHandle.value = null;
    emit("resizeEnd", currentLayout());
  };
  view.addEventListener("pointermove", onMove);
  view.addEventListener("pointerup", stop);
  view.addEventListener("pointercancel", stop);
  stopDrag = stop;
}

onMounted(() => {
  mounted.value = true;
});
onScopeDispose(() => stopDrag?.());

splitterContext.provide({
  collapsePanel,
  dir: dirState,
  disabled: disabledState,
  draggingHandle: computed(() => draggingHandle.value),
  expandPanel,
  getHandleIndex: (handleId) => handles.items.value.findIndex((item) => item.key === handleId),
  getPanelConstraints: (index) => constraints.value[index],
  getPanelId: (index) => panels.items.value[index]?.key,
  getPanelIndex: (panelId) => panels.items.value.findIndex((item) => item.key === panelId),
  getPanelSize: (panelId) => getSize(panels.items.value.findIndex((item) => item.key === panelId)),
  getSize,
  id: baseId,
  keyboardStep: keyboardStepState,
  orientation: orientationState,
  registerHandle: (input) =>
    handles.register({ key: input.id, value: input, element: input.element, order: input.order }),
  registerPanel: (input) =>
    panels.register({ key: input.id, value: input, element: input.element, order: input.order }),
  resizeHandle,
  setPanelSize,
  startDrag,
} satisfies SplitterContextValue);

type SplitterGroupSetupExpose = Omit<SplitterGroupExpose, "element" | "id" | "layout"> & {
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly layout: ComputedRef<SplitterLayout>;
};

const exposed = {
  element,
  id: baseId,
  layout: resolvedLayout,
  reset,
  setLayout,
} satisfies SplitterGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    :dir="dirState"
    :style="groupStyle"
    data-vize-ui="splitter-group"
    part="group"
    :data-state="state"
    :data-orientation="orientationState"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Only the flex layout that carries panel sizes is inline. */
</style>
