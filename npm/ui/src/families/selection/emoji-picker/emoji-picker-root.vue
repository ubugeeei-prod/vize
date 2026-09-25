<script setup lang="ts" generic="T">
import { computed, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { moveInGrid } from "../listbox-grid/listbox-grid-model.ts";
import { emojiPickerContext } from "./emoji-picker-context.ts";
import type { EmojiPickerContextValue } from "./emoji-picker-context.ts";
import {
  buildEmojiSections,
  containsEmojiFilter,
  createEmojiVirtualGrid,
  emojiGlyph,
  toSkinTone,
} from "./emoji-picker-model.ts";
import type { EmojiPickerAccessors, EmojiSkinTone } from "./emoji-picker-model.ts";
import type {
  EmojiPickerRootExpose,
  EmojiPickerRootProps,
  EmojiPickerSlotState,
} from "./emoji-picker-types.ts";

const {
  id = undefined,
  items,
  getEmoji,
  getName,
  getKeywords = undefined,
  getCategory = undefined,
  getSkins = undefined,
  categories = undefined,
  columns = 8,
  skinTone = undefined,
  defaultSkinTone = 0,
  search = undefined,
  filter = undefined,
  recent = undefined,
  recentLabel = "Recently used",
  searchLabel = "Search results",
  defaultLabel = "Emoji",
  dir = "ltr",
} = defineProps<EmojiPickerRootProps<T>>();

const emit = defineEmits<{
  /** Fired when an emoji is chosen, with the item and its glyph with the skin tone applied. */
  select: [item: T, glyph: string];

  /** Fired when the skin tone requests a new controlled value. */
  "update:skinTone": [tone: EmojiSkinTone];

  /** Fired when the search text requests a new controlled value. */
  "update:search": [text: string];
}>();

defineSlots<{
  /** Search, grid, categories, skin tones, and preview. Receives typed sections. */
  default?(props: EmojiPickerSlotState<T>): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "emoji-picker" });
const gridId = computed(() => deriveDeterministicId(baseId.value, "grid"));
const columnsState = computed(() =>
  Number.isFinite(columns) && columns >= 1 ? Math.floor(columns) : 1,
);
const toneState = useControllableState<EmojiSkinTone>({
  value: () => skinTone,
  defaultValue: () => toSkinTone(defaultSkinTone),
  onChange: (next) => emit("update:skinTone", next),
});
const searchState = useControllableState<string>({
  value: () => search,
  defaultValue: "",
  onChange: (next) => emit("update:search", next),
});
const accessors = computed<EmojiPickerAccessors<T>>(() => ({
  getCategory,
  getEmoji,
  getKeywords,
  getName,
  getSkins,
}));
const sections = computed(() =>
  buildEmojiSections<T>({
    accessors: accessors.value,
    categories,
    columns: columnsState.value,
    defaultLabel,
    filter: filter ?? containsEmojiFilter,
    items,
    recent,
    recentLabel,
    search: searchState.value.value,
    searchLabel,
  }),
);
const grid = computed(() => createEmojiVirtualGrid(sections.value, columnsState.value));
const activeState = shallowRef(-1);
const activeIndex = computed(() =>
  grid.value.cellAt(activeState.value) === undefined ? -1 : activeState.value,
);
const activeItem = computed<T | undefined>(() => {
  const cell = grid.value.cellAt(activeIndex.value);
  return cell === undefined ? undefined : sections.value[cell.section]?.items[cell.index];
});
const empty = computed(() => sections.value.length === 0);
const totalRows = computed(() => grid.value.rows + sections.value.length);
const gridFocus = new Set<() => void>();

function glyphOf(item: T): string {
  return emojiGlyph(item, toneState.value.value, accessors.value);
}

function currentGrid() {
  return grid.value;
}

function move(direction: Parameters<typeof moveInGrid>[0]): boolean {
  const layout = currentGrid();
  const target = moveInGrid(direction, {
    columns: columnsState.value,
    count: layout.count,
    direction: dir,
    index: activeIndex.value,
    isEnabled: (index) => layout.cellAt(index) !== undefined,
  });
  if (target === null) return false;
  activeState.value = target;
  return true;
}

function select(item: T, event: Event | null): void {
  void event;
  emit("select", item, glyphOf(item));
}

function currentActiveItem(): T | undefined {
  return activeItem.value;
}

function selectActive(event: Event): boolean {
  const item = currentActiveItem();
  if (item === undefined) return false;
  select(item, event);
  return true;
}

watch(
  () => searchState.value.value,
  () => {
    activeState.value = -1;
    if (searchState.value.value.length > 0) move("first");
  },
);

const context: EmojiPickerContextValue<T> = {
  activeIndex,
  activeItem,
  baseId,
  cellId: (section, index) => deriveDeterministicId(baseId.value, `cell-${section}-${index}`),
  columns: columnsState,
  empty,
  focusGrid: () => {
    for (const focus of gridFocus) focus();
  },
  glyphOf,
  gridId,
  labelId: (section) => deriveDeterministicId(baseId.value, `section-${section}`),
  move,
  nameOf: (item) => getName(item),
  registerGrid: (focus) => {
    gridFocus.add(focus);
    return () => {
      gridFocus.delete(focus);
    };
  },
  search: searchState.value,
  sections,
  select,
  selectActive,
  setActive: (index) => {
    if (currentGrid().cellAt(index) !== undefined) activeState.value = index;
  },
  setSearch: (text) => {
    searchState.set(text);
  },
  setSkinTone: (tone) => {
    toneState.set(toSkinTone(tone));
  },
  skinTone: toneState.value,
  totalRows,
  virtualIndex: (section, index) => currentGrid().indexOf(section, index),
};
emojiPickerContext.provide(context);

const slotState = computed<EmojiPickerSlotState<T>>(() => ({
  activeItem: activeItem.value,
  empty: empty.value,
  glyphOf,
  search: searchState.value.value,
  sections: sections.value,
  skinTone: toneState.value.value,
}));

type EmojiPickerRootSetupExpose = Omit<EmojiPickerRootExpose<T>, "activeItem" | "skinTone"> & {
  readonly activeItem: ComputedRef<T | undefined>;
  readonly skinTone: ComputedRef<EmojiSkinTone>;
};

const exposed = {
  activeItem,
  focusGrid: context.focusGrid,
  setSearch: context.setSearch,
  skinTone: toneState.value,
} satisfies EmojiPickerRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="emoji-picker"
    part="root"
    :dir
    :data-skin-tone="toneState.value.value"
    :data-empty="empty ? 'true' : undefined"
    :data-searching="searchState.value.value.length > 0 ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
