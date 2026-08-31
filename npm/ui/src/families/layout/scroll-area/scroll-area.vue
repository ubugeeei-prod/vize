<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { resolveScrollAreaAria, resolveScrollAreaLayout } from "./scroll-area-runtime.ts";
import type {
  ScrollAreaAriaState,
  ScrollAreaEmits,
  ScrollAreaExpose,
  ScrollAreaProps,
  ScrollAreaRootElement,
  ScrollAreaSlotState,
  ScrollAreaStyle,
} from "./scroll-area-types.ts";

const {
  ariaDescribedby = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  as = "div",
  blockSize = "auto",
  dir = "ltr",
  focusable = false,
  inlineSize = "auto",
  maxBlockSize = undefined,
  maxInlineSize = undefined,
  orientation = "vertical",
  overscrollBehavior = "auto",
  scrollBehavior = "auto",
  scrollbarGutter = "auto",
  scrollbarWidth = "auto",
} = defineProps<ScrollAreaProps>();

const emit = defineEmits<ScrollAreaEmits>();

defineSlots<{
  /** Renders scrollable content with current viewport, direction, and CSS-hook state. */
  default(props: ScrollAreaSlotState): unknown;
}>();

const root = useTemplateRef<ScrollAreaRootElement>("root");
const viewport = useTemplateRef<HTMLDivElement>("viewport");
const layout = computed(() =>
  resolveScrollAreaLayout({
    blockSize,
    dir,
    focusable,
    inlineSize,
    maxBlockSize,
    maxInlineSize,
    orientation,
    overscrollBehavior,
    scrollBehavior,
    scrollbarGutter,
    scrollbarWidth,
  }),
);
const ariaState = computed<ScrollAreaAriaState>(() =>
  resolveScrollAreaAria({ ariaDescribedby, ariaLabel, ariaLabelledby }),
);
const asState = computed(() => as);
const orientationState = computed(() => layout.value.orientation);
const directionState = computed(() => layout.value.dir);
const focusableState = computed(() => layout.value.focusable);
const blockSizeState = computed(() => layout.value.blockSize);
const inlineSizeState = computed(() => layout.value.inlineSize);
const maxBlockSizeState = computed(() => layout.value.maxBlockSize);
const maxInlineSizeState = computed(() => layout.value.maxInlineSize);
const overflowXState = computed(() => layout.value.overflowX);
const overflowYState = computed(() => layout.value.overflowY);
const overscrollBehaviorState = computed(() => layout.value.overscrollBehavior);
const scrollBehaviorState = computed(() => layout.value.scrollBehavior);
const scrollbarGutterState = computed(() => layout.value.scrollbarGutter);
const scrollbarWidthState = computed(() => layout.value.scrollbarWidth);
const scrollAreaState = computed(() => layout.value.state);
const scrollAreaStyle = computed<ScrollAreaStyle>(() => layout.value.style);
const ariaLabelState = computed(() => ariaState.value.ariaLabel);
const ariaLabelledbyState = computed(() => ariaState.value.ariaLabelledby);
const ariaDescribedbyState = computed(() => ariaState.value.ariaDescribedby);
const labelled = computed(
  () => ariaLabelState.value !== undefined || ariaLabelledbyState.value !== undefined,
);
const described = computed(() => ariaDescribedbyState.value !== undefined);
const viewportAriaAttributes = computed<{
  readonly "aria-describedby"?: string;
  readonly "aria-label"?: string;
  readonly "aria-labelledby"?: string;
  readonly role?: "region";
  readonly tabindex?: 0;
}>(() => {
  const attributes: {
    "aria-describedby"?: string;
    "aria-label"?: string;
    "aria-labelledby"?: string;
    role?: "region";
    tabindex?: 0;
  } = {};
  if (ariaDescribedbyState.value !== undefined) {
    attributes["aria-describedby"] = ariaDescribedbyState.value;
  }
  if (ariaLabelState.value !== undefined) attributes["aria-label"] = ariaLabelState.value;
  if (ariaLabelledbyState.value !== undefined) {
    attributes["aria-labelledby"] = ariaLabelledbyState.value;
  }
  if (labelled.value) attributes.role = "region";
  if (focusableState.value) attributes.tabindex = 0;
  return attributes;
});
const rootIntrinsicProps = computed(() => ({ style: scrollAreaStyle.value }));
const slotState = computed<ScrollAreaSlotState>(() => ({
  ariaDescribedby: ariaDescribedbyState.value,
  ariaLabel: ariaLabelState.value,
  ariaLabelledby: ariaLabelledbyState.value,
  as: asState.value,
  blockSize: blockSizeState.value,
  described: described.value,
  dir: directionState.value,
  focusable: focusableState.value,
  inlineSize: inlineSizeState.value,
  labelled: labelled.value,
  maxBlockSize: maxBlockSizeState.value,
  maxInlineSize: maxInlineSizeState.value,
  orientation: orientationState.value,
  overflowX: overflowXState.value,
  overflowY: overflowYState.value,
  overscrollBehavior: overscrollBehaviorState.value,
  scrollBehavior: scrollBehaviorState.value,
  scrollbarGutter: scrollbarGutterState.value,
  scrollbarWidth: scrollbarWidthState.value,
  state: scrollAreaState.value,
  style: scrollAreaStyle.value,
}));

function onScroll(event: Event): void {
  emit("scroll", event);
}

function focus(options?: FocusOptions): void {
  viewport.value?.focus(options);
}

function scrollTo(options: ScrollToOptions = {}): void {
  viewport.value?.scrollTo(options);
}

function scrollBy(options: ScrollToOptions = {}): void {
  viewport.value?.scrollBy(options);
}

type ScrollAreaSetupExpose = Omit<
  ScrollAreaExpose,
  | "ariaDescribedby"
  | "ariaLabel"
  | "ariaLabelledby"
  | "as"
  | "blockSize"
  | "described"
  | "dir"
  | "focusable"
  | "inlineSize"
  | "labelled"
  | "maxBlockSize"
  | "maxInlineSize"
  | "orientation"
  | "overflowX"
  | "overflowY"
  | "overscrollBehavior"
  | "root"
  | "scrollBehavior"
  | "scrollbarGutter"
  | "scrollbarWidth"
  | "state"
  | "style"
  | "viewport"
> & {
  readonly ariaDescribedby: ComputedRef<ScrollAreaExpose["ariaDescribedby"]>;
  readonly ariaLabel: ComputedRef<ScrollAreaExpose["ariaLabel"]>;
  readonly ariaLabelledby: ComputedRef<ScrollAreaExpose["ariaLabelledby"]>;
  readonly as: ComputedRef<ScrollAreaExpose["as"]>;
  readonly blockSize: ComputedRef<ScrollAreaExpose["blockSize"]>;
  readonly described: ComputedRef<ScrollAreaExpose["described"]>;
  readonly dir: ComputedRef<ScrollAreaExpose["dir"]>;
  readonly focusable: ComputedRef<ScrollAreaExpose["focusable"]>;
  readonly inlineSize: ComputedRef<ScrollAreaExpose["inlineSize"]>;
  readonly labelled: ComputedRef<ScrollAreaExpose["labelled"]>;
  readonly maxBlockSize: ComputedRef<ScrollAreaExpose["maxBlockSize"]>;
  readonly maxInlineSize: ComputedRef<ScrollAreaExpose["maxInlineSize"]>;
  readonly orientation: ComputedRef<ScrollAreaExpose["orientation"]>;
  readonly overflowX: ComputedRef<ScrollAreaExpose["overflowX"]>;
  readonly overflowY: ComputedRef<ScrollAreaExpose["overflowY"]>;
  readonly overscrollBehavior: ComputedRef<ScrollAreaExpose["overscrollBehavior"]>;
  readonly root: typeof root;
  readonly scrollBehavior: ComputedRef<ScrollAreaExpose["scrollBehavior"]>;
  readonly scrollbarGutter: ComputedRef<ScrollAreaExpose["scrollbarGutter"]>;
  readonly scrollbarWidth: ComputedRef<ScrollAreaExpose["scrollbarWidth"]>;
  readonly state: ComputedRef<ScrollAreaExpose["state"]>;
  readonly style: ComputedRef<ScrollAreaExpose["style"]>;
  readonly viewport: typeof viewport;
};

const exposed = {
  ariaDescribedby: ariaDescribedbyState,
  ariaLabel: ariaLabelState,
  ariaLabelledby: ariaLabelledbyState,
  as: asState,
  blockSize: blockSizeState,
  described,
  dir: directionState,
  focus,
  focusable: focusableState,
  inlineSize: inlineSizeState,
  labelled,
  maxBlockSize: maxBlockSizeState,
  maxInlineSize: maxInlineSizeState,
  orientation: orientationState,
  overflowX: overflowXState,
  overflowY: overflowYState,
  overscrollBehavior: overscrollBehaviorState,
  root,
  scrollBehavior: scrollBehaviorState,
  scrollBy,
  scrollbarGutter: scrollbarGutterState,
  scrollbarWidth: scrollbarWidthState,
  scrollTo,
  state: scrollAreaState,
  style: scrollAreaStyle,
  viewport,
} satisfies ScrollAreaSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="asState"
    ref="root"
    :dir="directionState"
    part="root"
    data-vize-ui="scroll-area"
    :data-state="scrollAreaState"
    :data-orientation="orientationState"
    :data-dir="directionState"
    :data-focusable="focusableState ? 'true' : 'false'"
    :data-overscroll-behavior="overscrollBehaviorState"
    :data-scroll-behavior="scrollBehaviorState"
    :data-scrollbar-gutter="scrollbarGutterState"
    :data-scrollbar-width="scrollbarWidthState"
    v-bind="rootIntrinsicProps"
  >
    <div
      ref="viewport"
      v-bind="viewportAriaAttributes"
      :dir="directionState"
      part="viewport"
      data-vize-ui="scroll-area-viewport"
      :data-state="scrollAreaState"
      :data-orientation="orientationState"
      :data-dir="directionState"
      :data-focusable="focusableState ? 'true' : 'false'"
      :data-overflow-x="overflowXState"
      :data-overflow-y="overflowYState"
      @scroll="onScroll"
    >
      <slot v-bind="slotState" />
    </div>
  </component>
</template>

<style scoped>
/* Headless by design. Native overflow CSS ships from scroll-area.css. */
</style>
