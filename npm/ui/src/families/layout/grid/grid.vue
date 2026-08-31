<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  GridAlign,
  GridAutoFlow,
  GridColumns,
  GridElement,
  GridExpose,
  GridGap,
  GridJustify,
  GridSlotState,
  GridStyle,
} from "./grid-types.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { resolveGridLayout } from "./grid-runtime.ts";

const {
  as = "div",
  align = "stretch",
  autoFlow = "row",
  columnGap,
  columns = 1,
  gap = 0,
  justify = "stretch",
  rowGap,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Native CSS `grid-template-columns` value. Numbers resolve to equal fr tracks.
   *
   * @default 1
   */
  readonly columns?: GridColumns;

  /**
   * Native CSS `gap` value between direct children. Numbers resolve to px lengths.
   *
   * @default 0
   */
  readonly gap?: GridGap;

  /**
   * Native CSS `row-gap` override. Numbers resolve to px lengths.
   *
   * @default undefined
   */
  readonly rowGap?: GridGap;

  /**
   * Native CSS `column-gap` override. Numbers resolve to px lengths.
   *
   * @default undefined
   */
  readonly columnGap?: GridGap;

  /**
   * Native CSS `align-items` value for grid items.
   *
   * @default "stretch"
   */
  readonly align?: GridAlign;

  /**
   * Native CSS `justify-items` value for grid items.
   *
   * @default "stretch"
   */
  readonly justify?: GridJustify;

  /**
   * Native CSS `grid-auto-flow` auto-placement mode.
   *
   * @default "row"
   */
  readonly autoFlow?: GridAutoFlow;
}>();

defineSlots<{
  /** Renders grid children with the resolved native grid state. */
  default(props: GridSlotState): unknown;
}>();

const element = useTemplateRef<GridElement>("element");
const layout = computed(() =>
  resolveGridLayout({ align, autoFlow, columnGap, columns, gap, justify, rowGap }),
);
const alignState = computed(() => layout.value.align);
const autoFlowState = computed(() => layout.value.autoFlow);
const columnGapState = computed(() => layout.value.columnGap);
const columnsState = computed(() => layout.value.columns);
const gapState = computed(() => layout.value.gap);
const justifyState = computed(() => layout.value.justify);
const rowGapState = computed(() => layout.value.rowGap);
const gridStyle = computed<GridStyle>(() => layout.value.style);
const slotState = computed<GridSlotState>(() => ({
  align: alignState.value,
  autoFlow: autoFlowState.value,
  columnGap: columnGapState.value,
  columns: columnsState.value,
  gap: gapState.value,
  justify: justifyState.value,
  rowGap: rowGapState.value,
  style: gridStyle.value,
}));
const intrinsicProps = computed(() => ({ style: gridStyle.value }));

type GridSetupExpose = Omit<
  GridExpose,
  | "align"
  | "autoFlow"
  | "columnGap"
  | "columns"
  | "element"
  | "gap"
  | "justify"
  | "rowGap"
  | "style"
> & {
  readonly align: typeof alignState;
  readonly autoFlow: typeof autoFlowState;
  readonly columnGap: typeof columnGapState;
  readonly columns: typeof columnsState;
  readonly element: typeof element;
  readonly gap: typeof gapState;
  readonly justify: typeof justifyState;
  readonly rowGap: typeof rowGapState;
  readonly style: typeof gridStyle;
};

const exposed = {
  align: alignState,
  autoFlow: autoFlowState,
  columnGap: columnGapState,
  columns: columnsState,
  element,
  gap: gapState,
  justify: justifyState,
  rowGap: rowGapState,
  style: gridStyle,
} satisfies GridSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="grid"
    :data-columns="columnsState"
    :data-auto-flow="autoFlowState"
    :data-align="alignState"
    :data-justify="justifyState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native grid layout is authored as intrinsic inline style. */
</style>
