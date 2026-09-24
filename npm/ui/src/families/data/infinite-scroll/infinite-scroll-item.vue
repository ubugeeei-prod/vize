<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { infiniteScrollContext } from "./infinite-scroll-context.ts";
import type {
  InfiniteScrollItemExpose,
  InfiniteScrollItemSlotState,
} from "./infinite-scroll-types.ts";

const {
  index,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Zero-based item position, announced one-based through `aria-posinset`. @default required */
  readonly index: number;

  /**
   * Ids of the article title, per the feed pattern.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Ids of the article's primary content.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Article content. Receives the announced position and set size. */
  default(props: InfiniteScrollItemSlotState): unknown;
}>();

const context = infiniteScrollContext.use();
const element = useTemplateRef<HTMLElement>("element");
const position = computed(() => index + 1);
const slotState = computed<InfiniteScrollItemSlotState>(() => ({
  position: position.value,
  setSize: context.setSize.value,
}));

type InfiniteScrollItemSetupExpose = Omit<
  InfiniteScrollItemExpose,
  keyof InfiniteScrollItemSlotState | "element"
> & {
  readonly element: typeof element;
  readonly position: ComputedRef<number>;
  readonly setSize: ComputedRef<number>;
};

const exposed = {
  element,
  position,
  setSize: context.setSize,
} satisfies InfiniteScrollItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <article
    ref="element"
    :tabindex="context.feed.value ? 0 : undefined"
    :aria-posinset="context.feed.value ? position : undefined"
    :aria-setsize="context.feed.value ? context.setSize.value : undefined"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="infinite-scroll-item"
    part="item"
    :data-position="position"
  >
    <slot v-bind="slotState" />
  </article>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
