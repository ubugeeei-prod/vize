<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useScrollSpy } from "../../interaction/scroll-spy/scroll-spy.ts";
import { tocContext } from "./toc-context.ts";
import type { TocContextValue } from "./toc-context.ts";
import type { TocRootExpose, TocScrollBehavior, TocSlotState, TocState } from "./toc-types.ts";

const {
  activeId = undefined,
  defaultActiveId = null,
  offset = 0,
  root = undefined,
  track = true,
  scrollBehavior = undefined,
  updateHash = true,
  ariaLabel = "Table of contents",
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Controlled active section id (`v-model:activeId`). `undefined` lets the scroll spy own it.
   *
   * @default undefined
   */
  readonly activeId?: string | null;

  /**
   * Active id rendered on the server and before the first measurement.
   *
   * @default null
   */
  readonly defaultActiveId?: string | null;

  /**
   * Activation line in CSS pixels below the scroll container top, such as a sticky header height.
   *
   * @default 0
   */
  readonly offset?: number;

  /**
   * Scroll container holding the sections. `undefined` tracks the document viewport.
   *
   * @default undefined
   */
  readonly root?: Element | null;

  /**
   * Track scroll position. Disable it to drive `activeId` entirely from outside.
   *
   * @default true
   */
  readonly track?: boolean;

  /**
   * Script-driven scrolling for link clicks. `undefined` keeps native fragment navigation.
   *
   * @default undefined
   */
  readonly scrollBehavior?: TocScrollBehavior;

  /**
   * Replace the URL fragment after script-driven scrolling.
   *
   * @default true
   */
  readonly updateHash?: boolean;

  /**
   * Accessible name of the navigation landmark.
   *
   * @default "Table of contents"
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the landmark instead of `ariaLabel`.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired with the section id that came into view or was navigated to. */
  "update:activeId": [id: string | null];

  /** Fired when a link is activated, before any script-driven scrolling. */
  navigate: [id: string, nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** TocList, TocItem, and TocLink children. Receives the active id and tracked ids. */
  default(props: TocSlotState): unknown;
}>();

const element = useTemplateRef<HTMLElement>("element");
const registry = createCollectionRegistry<string, string>();
const ids = computed(() => [...new Set(registry.items.value.map((item) => item.value))]);
const activeState = useControllableState<string | null>({
  value: () => activeId,
  defaultValue: () => defaultActiveId,
  onChange: (next) => emit("update:activeId", next),
});
function initialActiveId(): string | null {
  return defaultActiveId;
}

const spy = useScrollSpy({
  ids,
  initialActiveId: initialActiveId(),
  isDisabled: () => !track,
  offset: () => offset,
  root: () => root,
  onActiveChange: (next) => {
    activeState.set(next);
  },
});
const currentId = computed(() => activeState.value.value);
const state = computed<TocState>(() => (currentId.value === null ? "idle" : "active"));
const slotState = computed<TocSlotState>(() => ({
  activeId: currentId.value,
  ids: ids.value,
  state: state.value,
}));

function scrollTo(id: string): boolean {
  const moved = spy.scrollTo(
    id,
    scrollBehavior === undefined ? undefined : { behavior: scrollBehavior, block: "start" },
  );
  if (moved) activeState.set(id);
  return moved;
}

function navigate(targetId: string, event: MouseEvent): void {
  emit("navigate", targetId, event);
  if (event.defaultPrevented || scrollBehavior === undefined) return;
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
    return;
  }
  if (!scrollTo(targetId)) return;
  event.preventDefault();
  if (updateHash) {
    const view = element.value?.ownerDocument.defaultView;
    view?.history.replaceState(view.history.state, "", `#${encodeURIComponent(targetId)}`);
  }
}

tocContext.provide({
  activeId: currentId,
  navigate,
  registerLink: (input) =>
    registry.register({
      key: input.key,
      value: input.targetId,
      element: input.element,
    }),
} satisfies TocContextValue);

type TocRootSetupExpose = Omit<TocRootExpose, "activeId" | "element"> & {
  readonly activeId: ComputedRef<string | null>;
  readonly element: typeof element;
};

const exposed = {
  activeId: currentId,
  element,
  refresh: spy.refresh,
  scrollTo,
} satisfies TocRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <nav
    ref="element"
    :aria-label="ariaLabelledby === undefined ? ariaLabel : undefined"
    :aria-labelledby="ariaLabelledby"
    data-vize-ui="toc"
    part="root"
    :data-state="state"
    :data-active-id="currentId ?? undefined"
  >
    <slot v-bind="slotState" />
  </nav>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
