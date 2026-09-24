<script setup lang="ts" generic="Item">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef } from "vue";
import type { CSSProperties } from "vue";

import { computeMasonryLayout, visibleMasonryItems } from "./masonry-layout.ts";
import type { MasonryItemSlotState, MasonryKey } from "./masonry-types.ts";

const {
  items,
  columns = 3,
  gap = 16,
  estimateHeight = () => 200,
  getKey = (_item: Item, index: number) => index,
  virtualize = false,
  overscan = 400,
} = defineProps<{
  /** Items to lay out. @default required */
  readonly items: readonly Item[];

  /**
   * Number of columns.
   *
   * @default 3
   */
  readonly columns?: number;

  /**
   * Gap between items and columns in CSS pixels.
   *
   * @default 16
   */
  readonly gap?: number;

  /**
   * Height estimate used for the server-rendered (and first client) distribution
   * and for items that have not been measured yet.
   *
   * @default () => 200
   */
  readonly estimateHeight?: (item: Item, index: number) => number;

  /**
   * Stable key per item; measured heights are remembered by key.
   *
   * @default (item, index) => index
   */
  readonly getKey?: (item: Item, index: number) => MasonryKey;

  /**
   * Render only items near the viewport. The root becomes the scroll
   * container, so give it a height.
   *
   * @default false
   */
  readonly virtualize?: boolean;

  /**
   * Extra pixels rendered above and below the viewport when virtualizing.
   *
   * @default 400
   */
  readonly overscan?: number;
}>();

defineSlots<{
  /** Renders one item. Receives the item, its index, and its column. */
  item(props: MasonryItemSlotState<Item>): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const measured = shallowRef<ReadonlyMap<MasonryKey, number>>(new Map());
const scrollTop = shallowRef(0);
const viewportHeight = shallowRef<number | null>(null);
const columnCount = computed(() => (Number.isInteger(columns) && columns > 0 ? columns : 1));

const entries = computed(() =>
  items.map((item, index) => ({ item, index, key: getKey(item, index) })),
);
const heights = computed(() =>
  entries.value.map(
    ({ item, index, key }) => measured.value.get(key) ?? estimateHeight(item, index),
  ),
);
const layout = computed(() => computeMasonryLayout(heights.value, columnCount.value, gap));
const visible = computed<ReadonlySet<number> | null>(() => {
  if (!virtualize || viewportHeight.value === null) return null;
  return new Set(
    visibleMasonryItems(
      layout.value,
      scrollTop.value,
      scrollTop.value + viewportHeight.value,
      overscan,
    ),
  );
});

interface RenderedItem {
  readonly key: MasonryKey;
  readonly state: MasonryItemSlotState<Item>;
  readonly style: CSSProperties | undefined;
}

interface RenderedColumn {
  readonly id: string;
  readonly column: number;
  readonly items: readonly RenderedItem[];
}

const renderedColumns = computed<readonly RenderedColumn[]>(() =>
  layout.value.columns.map((indexes, column) => ({
    id: `column-${column}`,
    column,
    items: indexes.flatMap((index) => {
      const entry = entries.value[index];
      if (!entry) return [];
      return [{ key: entry.key, state: { item: entry.item, index, column }, style: undefined }];
    }),
  })),
);

const trackWidth = computed(
  () => `calc((100% - ${(columnCount.value - 1) * gap}px) / ${columnCount.value})`,
);
const renderedVirtual = computed<readonly RenderedItem[]>(() =>
  layout.value.placements.flatMap((placement) => {
    if (visible.value !== null && !visible.value.has(placement.index)) return [];
    const entry = entries.value[placement.index];
    if (!entry) return [];
    return [
      {
        key: entry.key,
        state: { item: entry.item, index: placement.index, column: placement.column },
        style: {
          position: "absolute",
          top: `${placement.top}px`,
          left: `calc(${trackWidth.value} * ${placement.column} + ${placement.column * gap}px)`,
          width: trackWidth.value,
        },
      },
    ];
  }),
);

function measure(): void {
  if (element.value) measureRoot(element.value);
}

function measureRoot(root: HTMLElement): void {
  const next = new Map(measured.value);
  let changed = false;
  for (const node of root.querySelectorAll<HTMLElement>("[data-masonry-index]")) {
    const entry = entries.value[Number(node.dataset["masonryIndex"])];
    if (!entry) continue;
    const height = node.getBoundingClientRect().height;
    if (!(height > 0)) continue;
    const key = entry.key;
    if (next.get(key) !== height) {
      next.set(key, height);
      changed = true;
    }
  }
  if (changed) measured.value = next;
  if (virtualize) {
    scrollTop.value = root.scrollTop;
    viewportHeight.value = root.clientHeight;
  }
}

let observer: ResizeObserver | null = null;
onMounted(() => {
  measure();
  if (typeof ResizeObserver === "function" && element.value) {
    observer = new ResizeObserver(() => measure());
    observer.observe(element.value);
  }
});
onScopeDispose(() => observer?.disconnect());

function onScroll(event: Event): void {
  if (!virtualize || !(event.currentTarget instanceof HTMLElement)) return;
  scrollTop.value = event.currentTarget.scrollTop;
  viewportHeight.value = event.currentTarget.clientHeight;
}

const rootStyle = computed<CSSProperties>(() =>
  virtualize
    ? { position: "relative", overflowY: "auto" }
    : { display: "flex", alignItems: "flex-start", gap: `${gap}px` },
);
const columnStyle = computed<CSSProperties>(() => ({
  display: "flex",
  flexDirection: "column",
  gap: `${gap}px`,
  flex: "1 1 0",
  minWidth: "0",
}));

defineExpose({ element, measure, layout });
</script>

<template>
  <div
    ref="element"
    data-vize-ui="masonry"
    :data-columns="columnCount"
    :data-virtualized="virtualize ? '' : undefined"
    :style="rootStyle"
    @scroll.passive="onScroll"
  >
    <template v-if="virtualize">
      <div data-part="sizer" :style="{ position: 'relative', height: `${layout.height}px` }">
        <div
          v-for="entry in renderedVirtual"
          :key="entry.key"
          data-part="item"
          :data-masonry-index="entry.state.index"
          :data-column="entry.state.column"
          :style="entry.style"
        >
          <slot name="item" v-bind="entry.state" />
        </div>
      </div>
    </template>
    <template v-else>
      <div
        v-for="column in renderedColumns"
        :key="column.id"
        data-part="column"
        :data-column="column.column"
        :style="columnStyle"
      >
        <div
          v-for="entry in column.items"
          :key="entry.key"
          data-part="item"
          :data-masonry-index="entry.state.index"
        >
          <slot name="item" v-bind="entry.state" />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
/* Headless by design. Column balancing is expressed as intrinsic inline layout. */
</style>
