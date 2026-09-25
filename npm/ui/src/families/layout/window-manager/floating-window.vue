<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, onScopeDispose, useTemplateRef, watch } from "vue";
import type { CSSProperties } from "vue";

import { windowManagerContext } from "./window-manager-context.ts";
import type { WindowEdge, WindowEntry, WindowMode, WindowRect } from "./window-manager-model.ts";
import type { FloatingWindowSlotState, WindowHandleProps } from "./window-manager-types.ts";

const {
  id,
  title,
  defaultRect = { x: 0, y: 0, width: 320, height: 240 },
  defaultMode = "normal",
  minWidth = 160,
  minHeight = 96,
  maxWidth = Number.POSITIVE_INFINITY,
  maxHeight = Number.POSITIVE_INFINITY,
  resizable = true,
} = defineProps<{
  /** Stable window id used in the layout. @default required */
  readonly id: string;

  /** Accessible window title, also shown by docks. @default required */
  readonly title: string;

  /**
   * Geometry used until the layout stores one for this window.
   *
   * @default { x: 0, y: 0, width: 320, height: 240 }
   */
  readonly defaultRect?: WindowRect;

  /**
   * Mode used until the layout stores one for this window.
   *
   * @default "normal"
   */
  readonly defaultMode?: WindowMode;

  /**
   * Minimum width in CSS pixels.
   *
   * @default 160
   */
  readonly minWidth?: number;

  /**
   * Minimum height in CSS pixels.
   *
   * @default 96
   */
  readonly minHeight?: number;

  /**
   * Maximum width in CSS pixels (also limited by the manager).
   *
   * @default Infinity
   */
  readonly maxWidth?: number;

  /**
   * Maximum height in CSS pixels (also limited by the manager).
   *
   * @default Infinity
   */
  readonly maxHeight?: number;

  /**
   * Render resize handles on every edge and corner.
   *
   * @default true
   */
  readonly resizable?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the slot's `close` action is used. The consumer removes the window. */
  close: [];
  /** Fired when the display mode changes. */
  modeChange: [mode: WindowMode];
}>();

defineSlots<{
  /** Window chrome and content. Receives state, handle props, and window actions. */
  default?(props: FloatingWindowSlotState): unknown;
}>();

const context = windowManagerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
// Inline `defaultRect` literals are new objects on every parent render; keep a
// structurally stable value so the manager does not re-render in a loop.
const stableDefaultRect = computed<WindowRect>((previous) =>
  previous &&
  previous.x === defaultRect.x &&
  previous.y === defaultRect.y &&
  previous.width === defaultRect.width &&
  previous.height === defaultRect.height
    ? previous
    : { x: defaultRect.x, y: defaultRect.y, width: defaultRect.width, height: defaultRect.height },
);
const unregister = context.register({
  id,
  title: () => title,
  defaultRect: () => stableDefaultRect.value,
  defaultMode: () => defaultMode,
});
onScopeDispose(unregister);

const entry = computed(() => context.entry(id));
const active = computed(() => context.activeId.value === id);
const constraints = computed(() => ({ minWidth, minHeight, maxWidth, maxHeight }));
const edges: readonly WindowEdge[] = ["n", "s", "e", "w", "ne", "nw", "se", "sw"];

watch(
  () => entry.value.mode,
  (mode) => emit("modeChange", mode),
);

function focus(): void {
  context.focus(id);
}

function track(
  event: PointerEvent,
  origin: WindowRect,
  apply: (deltaX: number, deltaY: number, origin: WindowRect) => void,
): void {
  if (event.button !== 0) return;
  const startX = event.clientX;
  const startY = event.clientY;
  const target =
    event.currentTarget instanceof Element ? event.currentTarget.ownerDocument : document;
  const onMove = (next: PointerEvent): void =>
    apply(next.clientX - startX, next.clientY - startY, origin);
  const onUp = (): void => {
    target.removeEventListener("pointermove", onMove);
    target.removeEventListener("pointerup", onUp);
    target.removeEventListener("pointercancel", onUp);
  };
  target.addEventListener("pointermove", onMove);
  target.addEventListener("pointerup", onUp);
  target.addEventListener("pointercancel", onUp);
  event.preventDefault();
}

const handleProps = computed<WindowHandleProps>(() => ({
  "data-part": "drag-handle",
  tabindex: 0,
  "aria-keyshortcuts":
    "ArrowUp ArrowDown ArrowLeft ArrowRight Shift+ArrowUp Shift+ArrowDown Shift+ArrowLeft Shift+ArrowRight",
  onPointerdown: (event) => {
    focus();
    track(event, entry.value, (deltaX, deltaY, origin) => context.move(id, deltaX, deltaY, origin));
  },
  onKeydown: (event) => {
    const step = context.keyboardStep.value;
    const delta: Readonly<Record<string, readonly [number, number]>> = {
      ArrowUp: [0, -step],
      ArrowDown: [0, step],
      ArrowLeft: [-step, 0],
      ArrowRight: [step, 0],
    };
    const move = delta[event.key];
    if (!move) return;
    event.preventDefault();
    if (event.shiftKey) {
      if (resizable) context.resize(id, "se", move[0], move[1], constraints.value);
    } else {
      context.move(id, move[0], move[1]);
    }
  },
}));

function onPointerDown(event: PointerEvent): void {
  focus();
  const handle = event.target instanceof Element ? event.target.closest("[data-edge]") : null;
  const edge = handle instanceof HTMLElement ? toEdge(handle.dataset["edge"]) : null;
  if (edge && resizable) {
    track(event, entry.value, (deltaX, deltaY, origin) =>
      context.resize(id, edge, deltaX, deltaY, constraints.value, origin),
    );
  }
}

function attach(node: HTMLDivElement): () => void {
  node.addEventListener("pointerdown", onPointerDown);
  node.addEventListener("focusin", focus);
  return () => {
    node.removeEventListener("pointerdown", onPointerDown);
    node.removeEventListener("focusin", focus);
  };
}

let detach = (): void => undefined;
onMounted(() => {
  if (element.value) detach = attach(element.value);
});
onBeforeUnmount(() => detach());

function toEdge(value: string | undefined): WindowEdge | null {
  return edges.find((edge) => edge === value) ?? null;
}

function styleFor(current: WindowEntry, zIndex: number): CSSProperties {
  if (current.mode === "maximized") {
    return { position: "absolute", zIndex, left: "0", top: "0", width: "100%", height: "100%" };
  }
  return {
    position: "absolute",
    zIndex,
    left: `${current.x}px`,
    top: `${current.y}px`,
    width: `${current.width}px`,
    height: `${current.height}px`,
  };
}

const windowStyle = computed<CSSProperties>(() => styleFor(entry.value, context.zIndex(id)));

function handleStyle(edge: WindowEdge): CSSProperties {
  const thickness = "6px";
  return {
    position: "absolute",
    top: edge.includes("n") ? "0" : edge.includes("s") ? `calc(100% - ${thickness})` : thickness,
    left: edge.includes("w") ? "0" : edge.includes("e") ? `calc(100% - ${thickness})` : thickness,
    width: edge === "n" || edge === "s" ? `calc(100% - 12px)` : thickness,
    height: edge === "e" || edge === "w" ? `calc(100% - 12px)` : thickness,
    cursor: `${edge}-resize`,
  };
}

const slotState = computed<FloatingWindowSlotState>(() => ({
  state: entry.value,
  active: active.value,
  handleProps: handleProps.value,
  minimize: () => context.setMode(id, "minimized"),
  maximize: () => context.setMode(id, "maximized"),
  restore: () => context.setMode(id, "normal"),
  toggleMaximize: () =>
    context.setMode(id, entry.value.mode === "maximized" ? "normal" : "maximized"),
  close: () => emit("close"),
}));

defineExpose({ element, state: entry, active, focus });
</script>

<template>
  <div
    ref="element"
    role="dialog"
    aria-modal="false"
    :aria-label="title"
    data-vize-ui="floating-window"
    :data-mode="entry.mode"
    :data-active="active ? '' : undefined"
    :hidden="entry.mode === 'minimized'"
    :style="windowStyle"
  >
    <slot v-bind="slotState" />
    <template v-if="resizable && entry.mode === 'normal'">
      <div
        v-for="edge in edges"
        :key="edge"
        data-part="resize-handle"
        :data-edge="edge"
        aria-hidden="true"
        :style="handleStyle(edge)"
      ></div>
    </template>
  </div>
</template>

<style scoped>
/* Headless by design. Geometry and stacking are intrinsic inline layout. */
</style>
