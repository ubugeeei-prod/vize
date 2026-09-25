<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { createVisibilityObserver } from "../../interaction/measure/measure.ts";
import type { VisibilityObserverController } from "../../interaction/measure/measure.ts";
import { isStickyStuck, stickyRootMargin } from "./sticky-geometry.ts";
import type { StickyExpose, StickySide, StickySlotState, StickyState } from "./sticky-types.ts";

const {
  as = "div",
  side = "top",
  offset = 0,
  root = null,
  disabled = false,
} = defineProps<{
  /**
   * Native element rendered as the sticky box, for example `header` or `nav`.
   *
   * @default "div"
   */
  readonly as?: keyof HTMLElementTagNameMap;

  /**
   * Edge the element pins to.
   *
   * @default "top"
   */
  readonly side?: StickySide;

  /**
   * Distance in CSS pixels between the pinned edge and the scroll container edge.
   *
   * @default 0
   */
  readonly offset?: number;

  /**
   * Scroll container used to detect the stuck state. `null` uses the viewport.
   *
   * @default null
   */
  readonly root?: HTMLElement | null;

  /**
   * Render in normal flow without `position: sticky` or stuck detection.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the element becomes pinned or releases. */
  "stuck-change": [stuck: boolean];
}>();

defineSlots<{
  /** Sticky contents. Receives the stuck state. */
  default(props: StickySlotState): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const stuck = shallowRef(false);
const offsetState = computed(() => (Number.isFinite(offset) ? offset : 0));
const sideState = computed(() => side);
const state = computed<StickyState>(() => (stuck.value ? "stuck" : "idle"));
const slotState = computed<StickySlotState>(() => ({
  side,
  state: state.value,
  stuck: stuck.value,
}));
const stickyStyle = computed(() =>
  disabled
    ? undefined
    : {
        "--vize-sticky-offset": `${offsetState.value}px`,
        position: "sticky" as const,
        [side]: `${offsetState.value}px`,
      },
);
let observer: VisibilityObserverController | null = null;
let mounted = false;

function containerEdges(target: HTMLElement): { readonly top: number; readonly bottom: number } {
  if (root) {
    const rect = root.getBoundingClientRect();
    return { bottom: rect.bottom, top: rect.top };
  }
  const view = target.ownerDocument.defaultView;
  return { bottom: view?.innerHeight ?? 0, top: 0 };
}

function setStuck(next: boolean): boolean {
  if (stuck.value !== next) {
    stuck.value = next;
    emit("stuck-change", next);
  }
  return next;
}

function refresh(): boolean {
  if (!mounted || disabled || !element.value) return setStuck(false);
  const rect = element.value.getBoundingClientRect();
  return setStuck(
    isStickyStuck(
      side,
      offsetState.value,
      { bottom: rect.bottom, top: rect.top },
      containerEdges(element.value),
    ),
  );
}

function disconnect(): void {
  observer?.dispose();
  observer = null;
}

function connect(): void {
  disconnect();
  if (!mounted || disabled || !element.value) {
    setStuck(false);
    return;
  }
  observer = createVisibilityObserver({
    root,
    rootMargin: stickyRootMargin(side, offsetState.value),
    threshold: [0, 1],
    onVisibilityChange: () => {
      refresh();
    },
  });
  observer.observe(element.value);
  refresh();
}

watch([element, sideState, offsetState, () => root, () => disabled], connect, {
  flush: "post",
});

onMounted(() => {
  mounted = true;
  connect();
});

onScopeDispose(() => {
  mounted = false;
  disconnect();
});

type StickySetupExpose = Omit<StickyExpose, "element" | "side" | "state" | "stuck"> & {
  readonly element: typeof element;
  readonly side: ComputedRef<StickySide>;
  readonly state: ComputedRef<StickyState>;
  readonly stuck: Readonly<ShallowRef<boolean>>;
};

const exposed = {
  element,
  refresh,
  side: sideState,
  state,
  stuck,
} satisfies StickySetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :style="stickyStyle"
    data-vize-ui="sticky"
    part="root"
    :data-side="side"
    :data-state="state"
    :data-stuck="stuck ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
