<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { splitterContext } from "./splitter-context.ts";
import type {
  SplitterPanelExpose,
  SplitterPanelSlotState,
  SplitterPanelState,
} from "./splitter-types.ts";

const {
  id = undefined,
  defaultSize = undefined,
  minSize = 0,
  maxSize = 100,
  collapsible = false,
  collapsedSize = 0,
  order = undefined,
} = defineProps<{
  /**
   * Consumer-owned panel id, also used by `aria-controls` on the adjacent handle.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Initial size in percent. Give every panel a size for exact server rendering.
   *
   * @default undefined
   */
  readonly defaultSize?: number;

  /**
   * Smallest expanded size in percent.
   *
   * @default 0
   */
  readonly minSize?: number;

  /**
   * Largest size in percent.
   *
   * @default 100
   */
  readonly maxSize?: number;

  /**
   * Allow the panel to snap to `collapsedSize` when dragged past half of `minSize`.
   *
   * @default false
   */
  readonly collapsible?: boolean;

  /**
   * Size in percent while collapsed.
   *
   * @default 0
   */
  readonly collapsedSize?: number;

  /**
   * Deterministic order for conditionally rendered panels.
   *
   * @default undefined
   */
  readonly order?: number;
}>();

const emit = defineEmits<{
  /** Fired when the panel snaps to its collapsed size. */
  collapse: [];

  /** Fired when a collapsed panel grows back to at least its minimum size. */
  expand: [];

  /** Fired with the panel's new size whenever it changes. */
  resize: [size: number, previous: number];
}>();

defineSlots<{
  /** Panel content. Receives the current size and collapsed state. */
  default(props: SplitterPanelSlotState): unknown;
}>();

const context = splitterContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const panelId = useDeterministicId({ id: () => id, hint: "splitter-panel" });
let registration: CollectionRegistration<string> | null = null;

watch(
  panelId,
  (next) => {
    registration?.unregister();
    registration = context.registerPanel({
      constraints: () => ({ collapsedSize, collapsible, maxSize, minSize }),
      defaultSize: () => defaultSize,
      element,
      id: next,
      order: () => order,
    });
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  registration?.unregister();
  registration = null;
});

const index = computed(() => context.getPanelIndex(panelId.value));
const size = computed(() => context.getPanelSize(panelId.value));
const collapsed = computed(() => collapsible && size.value <= collapsedSize);
const panelState = computed<SplitterPanelState>(() => (collapsed.value ? "collapsed" : "expanded"));
const panelStyle = computed(() => ({
  flexBasis: "0px",
  flexGrow: size.value,
  flexShrink: 1,
  overflow: "hidden",
}));
const slotState = computed<SplitterPanelSlotState>(() => ({
  collapsed: collapsed.value,
  index: index.value,
  size: size.value,
  state: panelState.value,
}));

watch(size, (next, previous) => emit("resize", next, previous));
watch(collapsed, (next) => {
  if (next) emit("collapse");
  else emit("expand");
});

type SplitterPanelSetupExpose = Omit<
  SplitterPanelExpose,
  "collapsed" | "element" | "id" | "size"
> & {
  readonly collapsed: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly size: ComputedRef<number>;
};

const exposed = {
  collapse: () => context.collapsePanel(index.value, "programmatic"),
  collapsed,
  element,
  expand: () => context.expandPanel(index.value, "programmatic"),
  id: panelId,
  resize: (next: number) => context.setPanelSize(index.value, next, "programmatic"),
  size,
} satisfies SplitterPanelSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="panelId"
    ref="element"
    :style="panelStyle"
    data-vize-ui="splitter-panel"
    part="panel"
    :data-state="panelState"
    :data-size="size"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Only the flex sizing that carries the layout is inline. */
</style>
