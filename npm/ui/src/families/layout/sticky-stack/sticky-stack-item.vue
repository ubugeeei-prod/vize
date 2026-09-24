<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef } from "vue";
import type { CSSProperties } from "vue";

import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { stickyStackContext } from "./sticky-stack-context.ts";
import type { StickyStackItemSlotState } from "./sticky-stack-types.ts";

const {
  as = "div",
  estimatedHeight = 0,
  disabled = false,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Height used before the item is measured (server rendering and hydration),
   * so items below get their offsets on first paint.
   *
   * @default 0
   */
  readonly estimatedHeight?: number;

  /**
   * Scroll normally instead of sticking; later items no longer stack below it.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the item becomes stuck (`true`) or scrolls free again (`false`). */
  stuckChange: [stuck: boolean];
}>();

defineSlots<{
  /** Item content. Receives the resolved offset and whether the item is stuck. */
  default?(props: StickyStackItemSlotState): unknown;
}>();

const context = stickyStackContext.use();
const element = useTemplateRef<HTMLElement>("element");
const measuredHeight = shallowRef<number | null>(null);
const stuck = shallowRef(false);
const id = Symbol("sticky-stack-item");
const unregister = context.register({
  id,
  element: () => element.value,
  height: () => measuredHeight.value ?? estimatedHeight,
  enabled: () => !disabled,
});
const top = computed(() => context.topOf(id));

function measure(): void {
  if (element.value) measureElement(element.value);
}

function measureElement(node: HTMLElement): void {
  const rect = node.getBoundingClientRect();
  // A zero height means "not laid out yet" (hidden ancestor, test DOM); keep the estimate.
  if (rect.height > 0) measuredHeight.value = rect.height;
  const next = !disabled && rect.height > 0 && rect.top <= top.value + 0.5;
  if (next !== stuck.value) {
    stuck.value = next;
    emit("stuckChange", next);
  }
}

let observer: ResizeObserver | null = null;
onMounted(() => {
  measure();
  window.addEventListener("scroll", measure, { capture: true, passive: true });
  if (typeof ResizeObserver === "function" && element.value) {
    observer = new ResizeObserver(() => measure());
    observer.observe(element.value);
  }
});
onScopeDispose(() => {
  unregister();
  observer?.disconnect();
  if (typeof window !== "undefined") {
    window.removeEventListener("scroll", measure, { capture: true });
  }
});

const itemStyle = computed<CSSProperties>(() =>
  disabled ? {} : { position: "sticky", top: `${top.value}px` },
);

defineExpose({ element, top, stuck, measure });
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="sticky-stack-item"
    :data-stuck="stuck ? '' : undefined"
    :data-disabled="disabled ? '' : undefined"
    :style="itemStyle"
  >
    <slot :top="top" :stuck="stuck" />
  </component>
</template>

<style scoped>
/* Headless by design. Sticky offsets are intrinsic inline layout. */
</style>
