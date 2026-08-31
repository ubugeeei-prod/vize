<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  ClusterAlign,
  ClusterElement,
  ClusterExpose,
  ClusterGap,
  ClusterJustify,
  ClusterSlotState,
  ClusterStyle,
} from "./cluster-types.ts";
import type { PrimitiveAs } from "../../foundations/primitive/primitive.ts";
import { resolveClusterLayout } from "./cluster-runtime.ts";

const {
  as = "div",
  gap = 0,
  align = "stretch",
  justify = "start",
  wrap = true,
  reversed = false,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Native CSS `gap` value between direct children. Numbers resolve to px lengths.
   *
   * @default 0
   */
  readonly gap?: ClusterGap;

  /**
   * Native CSS `align-items` value for the cross axis.
   *
   * @default "stretch"
   */
  readonly align?: ClusterAlign;

  /**
   * Native CSS `justify-content` value for the inline axis.
   *
   * @default "start"
   */
  readonly justify?: ClusterJustify;

  /**
   * Allow items to wrap onto additional lines.
   *
   * @default true
   */
  readonly wrap?: boolean;

  /**
   * Reverse inline item flow without changing DOM order.
   *
   * @default false
   */
  readonly reversed?: boolean;
}>();

defineSlots<{
  /** Renders cluster children with the resolved inline layout state. */
  default(props: ClusterSlotState): unknown;
}>();

const element = useTemplateRef<ClusterElement>("element");
const layout = computed(() => resolveClusterLayout({ align, gap, justify, reversed, wrap }));
const alignState = computed(() => layout.value.align);
const directionState = computed(() => layout.value.direction);
const gapState = computed(() => layout.value.gap);
const justifyState = computed(() => layout.value.justify);
const reversedState = computed(() => layout.value.reversed);
const clusterState = computed(() => layout.value.state);
const wrapState = computed(() => layout.value.wrap);
const wrapModeState = computed(() => layout.value.wrapMode);
const clusterStyle = computed<ClusterStyle>(() => layout.value.style);
const slotState = computed<ClusterSlotState>(() => ({
  align: alignState.value,
  direction: directionState.value,
  gap: gapState.value,
  justify: justifyState.value,
  reversed: reversedState.value,
  state: clusterState.value,
  wrap: wrapState.value,
  wrapMode: wrapModeState.value,
}));
const intrinsicProps = computed(() => ({ style: clusterStyle.value }));

type ClusterSetupExpose = Omit<
  ClusterExpose,
  "align" | "direction" | "element" | "gap" | "justify" | "reversed" | "state" | "wrap" | "wrapMode"
> & {
  readonly align: typeof alignState;
  readonly direction: typeof directionState;
  readonly element: typeof element;
  readonly gap: typeof gapState;
  readonly justify: typeof justifyState;
  readonly reversed: typeof reversedState;
  readonly state: typeof clusterState;
  readonly wrap: typeof wrapState;
  readonly wrapMode: typeof wrapModeState;
};

const exposed = {
  align: alignState,
  direction: directionState,
  element,
  gap: gapState,
  justify: justifyState,
  reversed: reversedState,
  state: clusterState,
  wrap: wrapState,
  wrapMode: wrapModeState,
} satisfies ClusterSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="cluster"
    :data-state="clusterState"
    :data-wrap="wrapState ? 'true' : 'false'"
    :data-reversed="reversedState ? 'true' : 'false'"
    :data-align="alignState"
    :data-justify="justifyState"
    :data-vize-cluster-direction="directionState"
    :data-vize-cluster-gap="gapState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native flex layout is authored as intrinsic inline style. */
</style>
