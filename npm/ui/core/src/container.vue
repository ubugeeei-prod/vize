<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  ContainerElement,
  ContainerExpose,
  ContainerLength,
  ContainerSize,
  ContainerSlotState,
  ContainerStyle,
} from "./container-types.ts";
import type { PrimitiveAs } from "./primitive.ts";
import { resolveContainerLayout } from "./container-runtime.ts";

const {
  as = "div",
  centered = true,
  maxInlineSize,
  paddingInline = 0,
  size = "md",
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;

  /**
   * Named max-inline-size preset.
   *
   * @default "md"
   */
  readonly size?: ContainerSize;

  /**
   * Native CSS `max-inline-size` override. Numbers resolve to px lengths.
   *
   * @default undefined
   */
  readonly maxInlineSize?: ContainerLength;

  /**
   * Native CSS `padding-inline` value. Numbers resolve to px lengths.
   *
   * @default 0
   */
  readonly paddingInline?: ContainerLength;

  /**
   * Center the host with logical inline auto margins.
   *
   * @default true
   */
  readonly centered?: boolean;
}>();

defineSlots<{
  /** Renders container children with the resolved logical sizing state. */
  default(props: ContainerSlotState): unknown;
}>();

const element = useTemplateRef<ContainerElement>("element");
const layout = computed(() =>
  resolveContainerLayout({ centered, maxInlineSize, paddingInline, size }),
);
const centeredState = computed(() => layout.value.centered);
const maxInlineSizeState = computed(() => layout.value.maxInlineSize);
const paddingInlineState = computed(() => layout.value.paddingInline);
const sizeState = computed(() => layout.value.size);
const containerStyle = computed<ContainerStyle>(() => layout.value.style);
const slotState = computed<ContainerSlotState>(() => ({
  centered: centeredState.value,
  maxInlineSize: maxInlineSizeState.value,
  paddingInline: paddingInlineState.value,
  size: sizeState.value,
  style: containerStyle.value,
}));
const intrinsicProps = computed(() => ({ style: containerStyle.value }));

type ContainerSetupExpose = Omit<
  ContainerExpose,
  "centered" | "element" | "maxInlineSize" | "paddingInline" | "size" | "style"
> & {
  readonly centered: typeof centeredState;
  readonly element: typeof element;
  readonly maxInlineSize: typeof maxInlineSizeState;
  readonly paddingInline: typeof paddingInlineState;
  readonly size: typeof sizeState;
  readonly style: typeof containerStyle;
};

const exposed = {
  centered: centeredState,
  element,
  maxInlineSize: maxInlineSizeState,
  paddingInline: paddingInlineState,
  size: sizeState,
  style: containerStyle,
} satisfies ContainerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="container"
    :data-size="sizeState"
    :data-centered="centeredState ? 'true' : 'false'"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native logical sizing is authored as intrinsic inline style. */
</style>
