<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs } from "../../../primitive.ts";
import type {
  SkeletonAriaState,
  SkeletonElement,
  SkeletonExpose,
  SkeletonSlotState,
  SkeletonState,
  SkeletonStyle,
} from "./skeleton-types.ts";

const {
  as = "div",
  loading = true,
  visible = true,
  ariaLabel = undefined,
  ariaHidden = undefined,
  blockSize = "1em",
  inlineSize = "100%",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether the placeholder represents pending content.
   *
   * @default true
   */
  readonly loading?: boolean;

  /**
   * Whether the placeholder remains rendered and visible in layout.
   *
   * @default true
   */
  readonly visible?: boolean;

  /**
   * Accessible status text when the skeleton should be announced.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Override the default accessibility policy. By default, labelled skeletons
   * are exposed as status regions and unlabelled skeletons are decorative.
   *
   * @default undefined
   */
  readonly ariaHidden?: boolean;

  /**
   * Value published to `--vize-ui-skeleton-block-size`.
   *
   * @default "1em"
   */
  readonly blockSize?: string;

  /**
   * Value published to `--vize-ui-skeleton-inline-size`.
   *
   * @default "100%"
   */
  readonly inlineSize?: string;
}>();

defineSlots<{
  /** Optional placeholder content. Receives current loading, visibility, and ARIA state. */
  default(props: SkeletonSlotState): unknown;
}>();

const element = useTemplateRef<SkeletonElement>("element");
const loadingState = computed(() => loading);
const visibleState = computed(() => visible);
const state = computed<SkeletonState>(() =>
  visibleState.value ? (loadingState.value ? "loading" : "loaded") : "hidden",
);
const hasAriaLabel = computed(() => ariaLabel != null && ariaLabel.length > 0);
const hiddenFromAssistiveTechnology = computed(() => ariaHidden ?? !hasAriaLabel.value);
const ariaState = computed<SkeletonAriaState>(() =>
  hiddenFromAssistiveTechnology.value ? "decorative" : "status",
);
const slotState = computed<SkeletonSlotState>(() => ({
  ariaState: ariaState.value,
  loading: loadingState.value,
  state: state.value,
  visible: visibleState.value,
}));
const skeletonStyle = computed<SkeletonStyle>(() => ({
  "--vize-ui-skeleton-block-size": blockSize,
  "--vize-ui-skeleton-inline-size": inlineSize,
}));
const intrinsicProps = computed(() => ({ style: skeletonStyle.value }));

type SkeletonSetupExpose = Omit<
  SkeletonExpose,
  "ariaState" | "element" | "loading" | "state" | "visible"
> & {
  readonly ariaState: typeof ariaState;
  readonly element: typeof element;
  readonly loading: typeof loadingState;
  readonly state: typeof state;
  readonly visible: typeof visibleState;
};

const exposed = {
  ariaState,
  element,
  loading: loadingState,
  state,
  visible: visibleState,
} satisfies SkeletonSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    :hidden="visible ? undefined : true"
    :role="ariaState === 'status' ? 'status' : undefined"
    :aria-hidden="ariaState === 'decorative' ? 'true' : undefined"
    :aria-label="ariaState === 'status' ? ariaLabel : undefined"
    data-vize-ui="skeleton"
    part="root"
    :data-state="state"
    :data-loading="loading ? 'true' : 'false'"
    :data-visible="visible ? 'true' : 'false'"
    :data-aria-state="ariaState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
