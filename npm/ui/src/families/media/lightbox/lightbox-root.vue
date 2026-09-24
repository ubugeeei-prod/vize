<script setup lang="ts" generic="Item">
import { computed, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import DialogRoot from "../../overlays/dialog/dialog-root.vue";
import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxContextValue } from "./lightbox-context.ts";
import {
  lightboxPreloadIndexes,
  resolveLightboxIndex,
  resolveLightboxMessages,
} from "./lightbox-state.ts";
import type {
  LightboxChangeReason,
  LightboxDirection,
  LightboxMessageOverrides,
  LightboxMessages,
  LightboxPartSlotState,
  LightboxRootExpose,
  LightboxSlotState,
  LightboxState,
} from "./lightbox-types.ts";

const {
  id = undefined,
  items,
  open = undefined,
  defaultOpen = false,
  index = undefined,
  defaultIndex = 0,
  loop = false,
  dir = "ltr",
  closeOnSwipeDown = true,
  preload = 1,
  getPreloadSrc = undefined,
  messages = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Items shown by the viewer; the item type flows into every root slot. @default required */
  readonly items: readonly Item[];

  /**
   * Controlled open state (`v-model:open`). `undefined` selects {@link defaultOpen}.
   *
   * @default undefined
   */
  readonly open?: boolean | undefined;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Controlled zero-based current index (`v-model:index`). `undefined` selects {@link defaultIndex}.
   *
   * @default undefined
   */
  readonly index?: number | undefined;

  /**
   * Initial index for uncontrolled use.
   *
   * @default 0
   */
  readonly defaultIndex?: number;

  /**
   * Wrap previous/next navigation at both ends.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Reading direction for arrow keys and swipes.
   *
   * @default "ltr"
   */
  readonly dir?: LightboxDirection;

  /**
   * Let a downward touch swipe close the viewer.
   *
   * @default true
   */
  readonly closeOnSwipeDown?: boolean;

  /**
   * Neighbours on each side warmed through {@link getPreloadSrc} while open.
   *
   * @default 1
   */
  readonly preload?: number;

  /**
   * Returns the image URL to warm for an item. Preloading runs on the client only.
   *
   * @default undefined
   */
  readonly getPreloadSrc?: ((item: Item) => string | undefined) | undefined;

  /**
   * Localized strings.
   *
   * @default undefined
   */
  readonly messages?: LightboxMessageOverrides | undefined;
}>();

const emit = defineEmits<{
  /** Fired when the open state requests a new controlled value. */
  "update:open": [open: boolean];

  /** Fired when the current index requests a new controlled value. */
  "update:index": [index: number];

  /** Fired after every distinct index request, with its cause. */
  change: [index: number, previous: number, reason: LightboxChangeReason];
}>();

defineSlots<{
  /** Triggers and the viewer content. Receives the inferred current item. */
  default?(props: LightboxSlotState<Item>): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "lightbox" });
const openState = useControllableState<boolean>({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const indexState = useControllableState<number>({
  value: () => index,
  defaultValue: () => defaultIndex,
});
const count = computed<number>(() => items.length);
const loopState = computed<boolean>(() => loop && count.value > 1);
const isOpen = computed<boolean>(() => openState.value.value);
const currentIndex = computed<number>(() =>
  resolveLightboxIndex(indexState.value.value, count.value, false),
);
const state = computed<LightboxState>(() => (isOpen.value ? "open" : "closed"));
const canGoPrevious = computed<boolean>(
  () => count.value > 1 && (loopState.value || currentIndex.value > 0),
);
const canGoNext = computed<boolean>(
  () => count.value > 1 && (loopState.value || currentIndex.value < count.value - 1),
);
const resolvedMessages = computed<LightboxMessages>(() => resolveLightboxMessages(messages));
const currentItem = computed<Item | undefined>(() => items[currentIndex.value]);
const partState = computed<LightboxPartSlotState>(() => ({
  canGoNext: canGoNext.value,
  canGoPrevious: canGoPrevious.value,
  count: count.value,
  index: currentIndex.value,
  open: isOpen.value,
  state: state.value,
}));
const slotState = computed<LightboxSlotState<Item>>(() => ({
  canGoNext: canGoNext.value,
  canGoPrevious: canGoPrevious.value,
  count: count.value,
  index: currentIndex.value,
  item: currentItem.value,
  items,
  open: isOpen.value,
  state: state.value,
}));
const thumbnails = new Map<number, HTMLButtonElement>();
const preloaded = new Set<string>();

function readIndex(): number {
  return currentIndex.value;
}

function goTo(requested: number, reason: LightboxChangeReason): boolean {
  const target = resolveLightboxIndex(requested, count.value, false);
  const previous = readIndex();
  if (target === previous || count.value === 0) return false;
  indexState.set(target);
  emit("update:index", target);
  emit("change", target, previous, reason);
  return true;
}

function step(delta: 1 | -1, reason: LightboxChangeReason): boolean {
  if (delta > 0 ? !canGoNext.value : !canGoPrevious.value) return false;
  return goTo(resolveLightboxIndex(readIndex() + delta, count.value, loopState.value), reason);
}

function setOpen(next: boolean): boolean {
  if (!openState.set(next)) return false;
  emit("update:open", next);
  return true;
}

function openAt(target: number): boolean {
  const moved = goTo(target, "trigger");
  return setOpen(true) || moved;
}

// Warm neighbouring images on the client only; watchers never run during SSR.
watch(
  [isOpen, currentIndex],
  () => {
    if (!isOpen.value || getPreloadSrc === undefined || typeof Image !== "function") return;
    for (const neighbour of lightboxPreloadIndexes(
      currentIndex.value,
      count.value,
      preload,
      loopState.value,
    )) {
      const item = items[neighbour];
      const source = item === undefined ? undefined : getPreloadSrc(item);
      if (source === undefined || preloaded.has(source)) continue;
      preloaded.add(source);
      const image = new Image();
      image.decoding = "async";
      image.src = source;
    }
  },
  { flush: "post" },
);

lightboxContext.provide({
  canGoNext,
  canGoPrevious,
  close: () => setOpen(false),
  closeOnSwipeDown: computed(() => closeOnSwipeDown),
  count,
  dir: computed(() => dir),
  focusThumbnail: (target) => thumbnails.get(target)?.focus(),
  getItemId: (target) => deriveDeterministicId(baseId.value, `item-${target}`),
  goTo,
  id: baseId,
  index: currentIndex,
  loop: loopState,
  messages: resolvedMessages,
  open: isOpen,
  openAt,
  registerThumbnail(target, thumbnail) {
    thumbnails.set(target, thumbnail);
    return () => {
      if (thumbnails.get(target) === thumbnail) thumbnails.delete(target);
    };
  },
  slotState: partState,
  state,
  step,
} satisfies LightboxContextValue);

type LightboxRootSetupExpose = Omit<LightboxRootExpose<Item>, keyof LightboxSlotState<Item>> & {
  readonly canGoNext: ComputedRef<boolean>;
  readonly canGoPrevious: ComputedRef<boolean>;
  readonly count: ComputedRef<number>;
  readonly index: ComputedRef<number>;
  readonly item: ComputedRef<Item | undefined>;
  readonly items: ComputedRef<readonly Item[]>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<LightboxState>;
};

const exposed = {
  canGoNext,
  canGoPrevious,
  close: () => setOpen(false),
  count,
  goTo: (target: number) => goTo(target, "api"),
  index: currentIndex,
  item: currentItem,
  items: computed<readonly Item[]>(() => items),
  next: () => step(1, "api"),
  open: isOpen,
  openAt: (target: number) => {
    const moved = goTo(target, "api");
    return setOpen(true) || moved;
  },
  previous: () => step(-1, "api"),
  state,
} satisfies LightboxRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="lightbox-root"
    part="root"
    :dir
    :data-state="state"
    :data-index="currentIndex"
    :data-count="count"
  >
    <DialogRoot :id="baseId" :open="isOpen" @update:open="setOpen">
      <slot v-bind="slotState" />
    </DialogRoot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
