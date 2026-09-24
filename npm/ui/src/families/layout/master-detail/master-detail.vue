<script setup lang="ts" generic="Key extends string | number">
import { computed, nextTick, useTemplateRef, watch } from "vue";
import type { CSSProperties } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type {
  MasterDetailDetailSlotState,
  MasterDetailEmptySlotState,
  MasterDetailLayout,
  MasterDetailMasterSlotState,
} from "./master-detail-types.ts";
import { useMasterDetailViewport } from "./master-detail-viewport.ts";

const {
  selected = undefined,
  defaultSelected = null,
  splitAt = 768,
  ssrWidth = undefined,
  masterSize = "minmax(16rem, 1fr)",
  detailSize = "2fr",
  masterLabel = undefined,
  detailLabel = undefined,
} = defineProps<{
  /**
   * Controlled selection (`v-model:selected`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly selected?: Key | null;

  /**
   * Initial uncontrolled selection.
   *
   * @default null
   */
  readonly defaultSelected?: Key | null;

  /**
   * Minimum viewport width in CSS pixels for the side-by-side layout.
   *
   * @default 768
   */
  readonly splitAt?: number;

  /**
   * Viewport width assumed during server rendering and hydration. Without it
   * the stacked layout is rendered until mount.
   *
   * @default undefined
   */
  readonly ssrWidth?: number;

  /**
   * CSS grid track of the master pane in split layout.
   *
   * @default "minmax(16rem, 1fr)"
   */
  readonly masterSize?: string;

  /**
   * CSS grid track of the detail pane in split layout.
   *
   * @default "2fr"
   */
  readonly detailSize?: string;

  /**
   * Accessible name of the master region.
   *
   * @default undefined
   */
  readonly masterLabel?: string;

  /**
   * Accessible name of the detail region.
   *
   * @default undefined
   */
  readonly detailLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired with the next selection (`null` when going back). */
  "update:selected": [selected: Key | null];
  /** Fired when the layout switches between split and stacked, including the post-mount switch away from the SSR layout. */
  layoutChange: [layout: MasterDetailLayout];
}>();

defineSlots<{
  /** The list. Receives the selection, layout, and a `select` action. */
  master(props: MasterDetailMasterSlotState<Key>): unknown;
  /** The selected item's detail. Receives the selection, layout, and a `back` action. */
  detail(props: MasterDetailDetailSlotState<Key>): unknown;
  /** Placeholder shown in split layout while nothing is selected. */
  empty?(props: MasterDetailEmptySlotState): unknown;
}>();

const masterElement = useTemplateRef<HTMLElement>("master");
const detailElement = useTemplateRef<HTMLElement>("detail");
const selection = useControllableState<Key | null>({
  value: () => selected,
  defaultValue: () => defaultSelected,
  onChange: (next) => emit("update:selected", next),
});
const viewportWidth = useMasterDetailViewport(() => ssrWidth);
const layout = computed<MasterDetailLayout>(() =>
  viewportWidth.value !== null && viewportWidth.value >= splitAt ? "split" : "stacked",
);
const current = computed(() => selection.value.value);
const showMaster = computed(() => layout.value === "split" || current.value === null);
const showDetail = computed(() => current.value !== null);

watch(layout, (next) => emit("layoutChange", next));

function select(key: Key): void {
  selection.set(key);
  if (layout.value === "stacked") void nextTick(() => detailElement.value?.focus());
}

function back(): void {
  selection.set(null);
  if (layout.value === "stacked") void nextTick(() => masterElement.value?.focus());
}

function onDetailKeydown(event: KeyboardEvent): void {
  if (event.key !== "Escape" || layout.value !== "stacked" || event.defaultPrevented) return;
  event.preventDefault();
  back();
}

watch(detailElement, (element, _previous, onCleanup) => {
  if (!element) return;
  element.addEventListener("keydown", onDetailKeydown);
  onCleanup(() => element.removeEventListener("keydown", onDetailKeydown));
});

const rootStyle = computed<CSSProperties>(() =>
  layout.value === "split"
    ? { display: "grid", gridTemplateColumns: `${masterSize} ${detailSize}` }
    : { display: "block" },
);

defineExpose({ layout, select, back, selected: current });
</script>

<template>
  <div data-vize-ui="master-detail" :data-layout="layout" :style="rootStyle">
    <section
      v-if="showMaster"
      ref="master"
      data-part="master"
      tabindex="-1"
      :aria-label="masterLabel"
    >
      <slot name="master" :selected="current" :layout="layout" :select="select" />
    </section>
    <section
      v-if="showDetail && current !== null"
      ref="detail"
      data-part="detail"
      tabindex="-1"
      :aria-label="detailLabel"
    >
      <slot name="detail" :selected="current" :layout="layout" :back="back" />
    </section>
    <section v-else-if="layout === 'split'" data-part="empty" :aria-label="detailLabel">
      <slot name="empty" :layout="layout" />
    </section>
  </div>
</template>

<style scoped>
/* Headless by design. Split and stacked layouts are intrinsic inline grid. */
</style>
