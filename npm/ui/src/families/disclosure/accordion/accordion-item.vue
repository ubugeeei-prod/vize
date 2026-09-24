<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { collapsibleContext } from "../collapsible/collapsible.ts";
import { accordionContext, accordionItemContext } from "./accordion-context.ts";
import type {
  AccordionItemExpose,
  AccordionItemSlotState,
  AccordionItemState,
  AccordionValue,
} from "./accordion-types.ts";

const { value, disabled = false } = defineProps<{
  /**
   * Item identity used by the Accordion model.
   *
   * @default required
   */
  readonly value: AccordionValue;

  /**
   * Disable user activation of this item while preserving its open state.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** AccordionHeader and AccordionContent children. Receives the item state. */
  default(props: AccordionItemSlotState): unknown;
}>();

const context = accordionContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const valueState = computed(() => value);
let unregister: (() => void) | null = null;

watch(
  () => value,
  (next) => {
    unregister?.();
    unregister = context.registerItem(next);
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  unregister?.();
  unregister = null;
});
const itemId = computed(() => context.getItemId(value));
const triggerId = computed(() => deriveDeterministicId(itemId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(itemId.value, "content"));
const open = computed(() => context.isOpen(value));
const disabledState = computed(() => context.disabled.value || disabled);
const locked = computed(() => context.isLocked(value));
const state = computed<AccordionItemState>(() => (open.value ? "open" : "closed"));
const slotState = computed<AccordionItemSlotState>(() => ({
  disabled: disabledState.value,
  locked: locked.value,
  open: open.value,
  state: state.value,
  value,
}));

function setOpen(next: boolean, nativeEvent: Event | null = null): boolean {
  return context.setItemOpen(value, next, nativeEvent);
}

const expand = (nativeEvent: Event | null = null) => setOpen(true, nativeEvent);
const collapse = (nativeEvent: Event | null = null) => setOpen(false, nativeEvent);
const toggle = (nativeEvent: Event | null = null) => setOpen(!open.value, nativeEvent);

// Each item publishes the Collapsible contract, so CollapsibleTrigger and
// CollapsibleContent also work inside an AccordionItem.
collapsibleContext.provide({
  collapse,
  contentId,
  disabled: disabledState,
  expand,
  id: itemId,
  open,
  setOpen,
  state,
  toggle,
  triggerId,
});
accordionItemContext.provide({ locked, value: valueState });

type AccordionItemSetupExpose = Omit<
  AccordionItemExpose,
  "contentId" | "disabled" | "element" | "locked" | "open" | "state" | "triggerId" | "value"
> & {
  readonly contentId: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly locked: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<AccordionItemState>;
  readonly triggerId: ComputedRef<string>;
  readonly value: ComputedRef<AccordionValue>;
};

const exposed = {
  collapse,
  contentId,
  disabled: disabledState,
  element,
  expand,
  locked,
  open,
  state,
  toggle,
  triggerId,
  value: valueState,
} satisfies AccordionItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="itemId"
    ref="element"
    data-vize-ui="accordion-item"
    part="item"
    :data-state="state"
    :data-orientation="context.orientation.value"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
