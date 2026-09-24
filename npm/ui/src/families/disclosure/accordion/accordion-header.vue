<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import { collapsibleContext } from "../collapsible/collapsible.ts";
import { accordionContext, accordionItemContext } from "./accordion-context.ts";
import type {
  AccordionHeaderExpose,
  AccordionHeadingLevel,
  AccordionItemSlotState,
} from "./accordion-types.ts";

const { level = undefined, as = undefined } = defineProps<{
  /**
   * Native heading level. `undefined` uses the root `headingLevel`.
   *
   * @default undefined
   */
  readonly level?: AccordionHeadingLevel;

  /**
   * Element or component to render instead of the native `h${level}`.
   *
   * @default undefined
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Heading contents, normally one AccordionTrigger. Receives the item state. */
  default(props: AccordionItemSlotState): unknown;
}>();

const context = accordionContext.use();
const item = accordionItemContext.use();
const collapsible = collapsibleContext.use();
const element = useTemplateRef<PrimitiveElement>("element");
const levelState = computed(() => level ?? context.headingLevel.value);
const resolvedAs = computed<PrimitiveAs>(() => as ?? `h${levelState.value}`);

type AccordionHeaderSetupExpose = Omit<AccordionHeaderExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
} satisfies AccordionHeaderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="resolvedAs"
    ref="element"
    data-vize-ui="accordion-header"
    part="header"
    :data-level="levelState"
    :data-state="collapsible.state.value"
    :data-orientation="context.orientation.value"
    :data-disabled="collapsible.disabled.value ? 'true' : undefined"
  >
    <slot
      :disabled="collapsible.disabled.value"
      :locked="item.locked.value"
      :open="collapsible.open.value"
      :state="collapsible.state.value"
      :value="item.value.value"
    />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
