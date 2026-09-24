<script setup lang="ts" generic="T">
import { computed, onScopeDispose, useTemplateRef, watch, watchEffect } from "vue";

import { useVirtualizer } from "../../interaction/virtualizer/virtualizer.ts";
import { selectContext } from "./select-context.ts";
import type { SelectVirtualItemSlotState } from "./select-types.ts";

const {
  items,
  estimateItemSize = 32,
  overscan = 4,
  initialViewportHeight = 256,
  getItemKey = undefined,
} = defineProps<{
  /** Every option value in display order; only the visible window is rendered. @default required */
  readonly items: readonly T[];

  /**
   * Estimated option height in CSS pixels used until options are measured.
   *
   * @default 32
   */
  readonly estimateItemSize?: number;

  /**
   * Options rendered beyond each edge of the visible window.
   *
   * @default 4
   */
  readonly overscan?: number;

  /**
   * Viewport height assumed for server rendering and before measurement.
   *
   * @default 256
   */
  readonly initialViewportHeight?: number;

  /**
   * Stable render key for an option. Defaults to its index.
   *
   * @default undefined
   */
  readonly getItemKey?: (item: T, index: number) => string | number;
}>();

defineSlots<{
  /** Render one `SelectItem` with `:value="item"` and `:index="index"`. */
  default?(props: SelectVirtualItemSlotState<T>): unknown;
}>();

const context = selectContext.use();
const spacer = useTemplateRef<HTMLDivElement>("spacer");
const windowElement = useTemplateRef<HTMLDivElement>("windowElement");
// The initial rect only seeds server and pre-measure layout; later changes are
// superseded by the measured viewport, so it is read once by design.
function readInitialHeight(): number {
  return initialViewportHeight;
}

const virtualizer = useVirtualizer({
  count: () => items.length,
  estimateItemSize: () => estimateItemSize,
  getItemKey: (index) => {
    const item = items[index];
    return item === undefined || getItemKey === undefined ? index : getItemKey(item, index);
  },
  initialRect: { height: readInitialHeight(), width: 0 },
  overscan: () => overscan,
});
/** One rendered window row: its render key and typed slot props. */
interface VirtualRow {
  readonly key: string | number;
  readonly props: SelectVirtualItemSlotState<T>;
}

const rows = computed<readonly VirtualRow[]>(() =>
  virtualizer.virtualItems.value.flatMap((virtualItem) => {
    const item = items[virtualItem.index];
    return item === undefined
      ? []
      : [{ key: virtualItem.key, props: { index: virtualItem.index, item } }];
  }),
);
const offset = computed(() => virtualizer.virtualItems.value[0]?.start ?? 0);

// Attaching a viewport adopts its current scroll offset, so scroll requests
// made before the viewport exists are replayed once it is attached.
let attached = false;
let pendingScroll: number | null = null;

function scrollToIndex(index: number): void {
  if (index < 0 || index >= items.length) return;
  if (!attached) {
    pendingScroll = index;
    return;
  }
  virtualizer.scrollToIndex(index, "auto");
}

const release = context.setVirtualAdapter({
  count: () => items.length,
  disabledAt: (index) => {
    const item = items[index];
    return item !== undefined && context.isValueDisabled(item);
  },
  scrollToIndex,
  textAt: (index) => {
    const item = items[index];
    return item === undefined ? "" : context.textOf(item);
  },
});
onScopeDispose(release);

watch(
  () => context.viewportElement.value ?? context.contentElement.value,
  (viewport) => {
    virtualizer.setViewport(viewport);
    attached = viewport !== null;
    const pending = pendingScroll;
    if (attached && pending !== null) {
      pendingScroll = null;
      virtualizer.scrollToIndex(pending, "auto");
    }
  },
  { flush: "post", immediate: true },
);

// Geometry is measurement-driven output, so it is applied imperatively like
// the positioner host rather than through template style bindings.
watchEffect(
  () => {
    if (spacer.value !== null) {
      spacer.value.style.cssText = `position:relative;block-size:${virtualizer.totalSize.value}px;`;
    }
    if (windowElement.value !== null) {
      windowElement.value.style.cssText = `position:absolute;inset-inline:0;inset-block-start:0;transform:translateY(${offset.value}px);`;
    }
  },
  { flush: "post" },
);

function windowHost(): HTMLDivElement | null {
  return windowElement.value;
}

watch(
  rows,
  () => {
    const host = windowHost();
    if (host === null) return;
    for (const child of host.querySelectorAll("[data-index]")) {
      const index = Number(child.getAttribute("data-index"));
      if (Number.isInteger(index) && index >= 0 && index < items.length) {
        virtualizer.measureElement(child, index);
      }
    }
  },
  { flush: "post" },
);

defineExpose({
  scrollToIndex,
  totalSize: virtualizer.totalSize,
  virtualItems: virtualizer.virtualItems,
});
</script>

<template>
  <div
    ref="spacer"
    role="presentation"
    :data-vize-ui="`${context.partPrefix}-virtualizer`"
    part="virtualizer"
    :data-count="items.length"
  >
    <div ref="windowElement" role="presentation" data-vize-ui="select-virtualizer-window">
      <template v-for="row in rows as readonly VirtualRow[]" :key="row.key">
        <slot v-bind="row.props" />
      </template>
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
