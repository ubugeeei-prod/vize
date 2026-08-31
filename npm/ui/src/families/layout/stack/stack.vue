<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { resolveStackLayout } from "./stack-runtime.ts";
import type {
  StackAlign,
  StackAxis,
  StackElement,
  StackExpose,
  StackGap,
  StackJustify,
  StackSlotState,
  StackStyle,
} from "./stack-types.ts";

const {
  as = "div",
  axis = "block",
  reversed = false,
  gap = "1rem",
  align = "stretch",
  justify = "start",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Logical flex main axis. `block` stacks vertically in the current writing mode;
   * `inline` follows the document direction, including RTL.
   *
   * @default "block"
   */
  readonly axis?: StackAxis;

  /**
   * Reverse the selected logical main axis without changing DOM order.
   *
   * @default false
   */
  readonly reversed?: boolean;

  /**
   * Native CSS `gap` value between direct children.
   *
   * @default "1rem"
   */
  readonly gap?: StackGap;

  /**
   * Native CSS `align-items` value for the cross axis.
   *
   * @default "stretch"
   */
  readonly align?: StackAlign;

  /**
   * Native CSS `justify-content` value for the main axis.
   *
   * @default "start"
   */
  readonly justify?: StackJustify;
}>();

defineSlots<{
  /** Renders stack children with the resolved logical layout state. */
  default(props: StackSlotState): unknown;
}>();

const element = useTemplateRef<StackElement>("element");
const layout = computed(() => resolveStackLayout({ align, axis, gap, justify, reversed }));
const alignState = computed(() => layout.value.align);
const axisState = computed(() => layout.value.axis);
const directionState = computed(() => layout.value.direction);
const gapState = computed(() => layout.value.gap);
const justifyState = computed(() => layout.value.justify);
const reversedState = computed(() => layout.value.reversed);
const stackState = computed(() => layout.value.state);
const stackStyle = computed<StackStyle>(() => layout.value.style);
const slotState = computed<StackSlotState>(() => ({
  align: alignState.value,
  axis: axisState.value,
  direction: directionState.value,
  gap: gapState.value,
  justify: justifyState.value,
  reversed: reversedState.value,
  state: stackState.value,
}));
const intrinsicProps = computed(() => ({ style: stackStyle.value }));

type StackSetupExpose = Omit<
  StackExpose,
  "align" | "axis" | "direction" | "element" | "gap" | "justify" | "reversed" | "state"
> & {
  readonly align: typeof alignState;
  readonly axis: typeof axisState;
  readonly direction: typeof directionState;
  readonly element: typeof element;
  readonly gap: typeof gapState;
  readonly justify: typeof justifyState;
  readonly reversed: typeof reversedState;
  readonly state: typeof stackState;
};

const exposed = {
  align: alignState,
  axis: axisState,
  direction: directionState,
  element,
  gap: gapState,
  justify: justifyState,
  reversed: reversedState,
  state: stackState,
} satisfies StackSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="stack"
    :data-state="stackState"
    :data-axis="axisState"
    :data-reversed="reversedState ? 'true' : 'false'"
    :data-vize-stack-direction="directionState"
    :data-vize-stack-gap="gapState"
    :data-vize-stack-align="alignState"
    :data-vize-stack-justify="justifyState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native flex layout is authored as intrinsic inline style. */
</style>
