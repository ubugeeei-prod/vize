<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, triggerRef, useTemplateRef } from "vue";
import type { CSSProperties } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { windowManagerContext } from "./window-manager-context.ts";
import type { WindowRegistration } from "./window-manager-context.ts";
import {
  bringWindowToFront,
  emptyWindowLayout,
  moveWindowRect,
  resizeWindowRect,
} from "./window-manager-model.ts";
import type {
  WindowBounds,
  WindowEntry,
  WindowLayout,
  WindowMode,
  WindowRect,
} from "./window-manager-model.ts";
import type { WindowManagerSlotState, WindowSummary } from "./window-manager-types.ts";

const {
  layout = undefined,
  defaultLayout = emptyWindowLayout,
  snapThreshold = 8,
  keyboardStep = 16,
  baseZIndex = 1,
} = defineProps<{
  /**
   * Controlled layout (`v-model:layout`). `undefined` selects uncontrolled
   * behavior, which lets `useWindowLayoutPersistence` load a stored layout after mount.
   *
   * @default undefined
   */
  readonly layout?: WindowLayout;

  /**
   * Initial uncontrolled layout. Windows missing from it use their own defaults.
   *
   * @default emptyWindowLayout
   */
  readonly defaultLayout?: WindowLayout;

  /**
   * Distance in CSS pixels at which moving windows snap to edges; `0` disables snapping.
   *
   * @default 8
   */
  readonly snapThreshold?: number;

  /**
   * Pixels moved or resized by one arrow key press on a drag handle.
   *
   * @default 16
   */
  readonly keyboardStep?: number;

  /**
   * `z-index` of the backmost window; later windows stack above it.
   *
   * @default 1
   */
  readonly baseZIndex?: number;
}>();

const emit = defineEmits<{
  /** Fired with the next layout after any move, resize, focus, or mode change. */
  "update:layout": [layout: WindowLayout];
}>();

defineSlots<{
  /** FloatingWindow and WindowDock children. Receives the active id and window summaries. */
  default?(props: WindowManagerSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const state = useControllableState<WindowLayout>({
  value: () => layout,
  defaultValue: () => defaultLayout,
  onChange: (next) => emit("update:layout", next),
});
const registrations = shallowRef<WindowRegistration[]>([]);
const size = shallowRef<WindowBounds | null>(null);

function registrationOf(id: string): WindowRegistration | undefined {
  return registrations.value.find((registration) => registration.id === id);
}

function entryOf(id: string): WindowEntry {
  const stored = state.value.value.windows[id];
  if (stored) return stored;
  const registration = registrationOf(id);
  const rect = registration?.defaultRect() ?? { x: 0, y: 0, width: 320, height: 240 };
  return { ...rect, mode: registration?.defaultMode() ?? "normal" };
}

const stackOrder = computed<readonly string[]>(() => {
  const order = state.value.value.order;
  const pending = registrations.value
    .map((registration) => registration.id)
    .filter((id) => !order.includes(id));
  return [...order, ...pending];
});
const activeId = computed(() => {
  for (let index = stackOrder.value.length - 1; index >= 0; index -= 1) {
    const id = stackOrder.value[index];
    if (id !== undefined && registrationOf(id) && entryOf(id).mode !== "minimized") return id;
  }
  return null;
});

function commit(id: string, entry: WindowEntry, raise: boolean): void {
  const current = state.value.value;
  const order = raise ? bringWindowToFront(stackOrder.value, id) : stackOrder.value;
  state.set({ windows: { ...current.windows, [id]: entry }, order });
}

function visibleOthers(id: string): WindowRect[] {
  return registrations.value
    .filter((registration) => registration.id !== id)
    .map((registration) => entryOf(registration.id))
    .filter((entry) => entry.mode === "normal");
}

const windows = computed<readonly WindowSummary[]>((previous) => {
  const next = registrations.value.map((registration) => ({
    id: registration.id,
    title: registration.title(),
    mode: entryOf(registration.id).mode,
    active: activeId.value === registration.id,
    activate: () => {
      const entry = entryOf(registration.id);
      commit(
        registration.id,
        entry.mode === "minimized" ? { ...entry, mode: "normal" } : entry,
        true,
      );
    },
    minimize: () =>
      commit(registration.id, { ...entryOf(registration.id), mode: "minimized" }, false),
  }));
  const unchanged =
    previous !== undefined &&
    previous.length === next.length &&
    previous.every(
      (summary, index) =>
        summary.id === next[index]?.id &&
        summary.title === next[index]?.title &&
        summary.mode === next[index]?.mode &&
        summary.active === next[index]?.active,
    );
  return unchanged ? previous : next;
});

windowManagerContext.provide({
  register: (registration) => {
    registrations.value.push(registration);
    triggerRef(registrations);
    return () => {
      const index = registrations.value.indexOf(registration);
      if (index === -1) return;
      registrations.value.splice(index, 1);
      triggerRef(registrations);
    };
  },
  entry: entryOf,
  zIndex: (id) => baseZIndex + Math.max(0, stackOrder.value.indexOf(id)),
  activeId,
  bounds: computed(() => size.value),
  windows,
  keyboardStep: computed(() => keyboardStep),
  focus: (id) => {
    if (stackOrder.value.at(-1) === id && state.value.value.order.includes(id)) return;
    commit(id, entryOf(id), true);
  },
  move: (id, deltaX, deltaY, origin) => {
    const entry = entryOf(id);
    if (entry.mode !== "normal") return;
    const rect = moveWindowRect(
      origin ?? entry,
      deltaX,
      deltaY,
      visibleOthers(id),
      size.value,
      snapThreshold,
    );
    commit(id, { ...rect, mode: entry.mode }, true);
  },
  resize: (id, edge, deltaX, deltaY, constraints, origin) => {
    const entry = entryOf(id);
    if (entry.mode !== "normal") return;
    const rect = resizeWindowRect(origin ?? entry, edge, deltaX, deltaY, constraints, size.value);
    commit(id, { ...rect, mode: entry.mode }, true);
  },
  setMode: (id, mode: WindowMode) => commit(id, { ...entryOf(id), mode }, mode !== "minimized"),
});

let observer: ResizeObserver | null = null;
function measure(): void {
  if (!element.value) return;
  const { clientWidth, clientHeight } = element.value;
  size.value =
    clientWidth > 0 && clientHeight > 0 ? { width: clientWidth, height: clientHeight } : null;
}
onMounted(() => {
  measure();
  if (typeof ResizeObserver === "function" && element.value) {
    observer = new ResizeObserver(measure);
    observer.observe(element.value);
  }
});
onScopeDispose(() => observer?.disconnect());

const rootStyle: CSSProperties = { position: "relative", overflow: "hidden" };

defineExpose({ element, layout: state.value, activeId, measure });
</script>

<template>
  <div ref="element" data-vize-ui="window-manager" :style="rootStyle">
    <slot :active-id="activeId" :windows="windows" />
  </div>
</template>

<style scoped>
/* Headless by design. The manager is the positioning context for its windows. */
</style>
