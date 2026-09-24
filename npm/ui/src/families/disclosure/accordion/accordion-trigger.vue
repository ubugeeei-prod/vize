<script setup lang="ts">
import { onUnmounted, useTemplateRef, watch } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { collapsibleContext } from "../collapsible/collapsible.ts";
import { accordionContext, accordionItemContext } from "./accordion-context.ts";
import type {
  AccordionItemSlotState,
  AccordionTriggerExpose,
  AccordionValue,
} from "./accordion-types.ts";

const { ariaLabel = undefined, ariaLabelledby = undefined } = defineProps<{
  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the trigger.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired before the trigger requests a toggle. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger contents. Receives the item state. */
  default(props: AccordionItemSlotState): unknown;
}>();

const context = accordionContext.use();
const item = accordionItemContext.use();
const collapsible = collapsibleContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
let registration: CollectionRegistration<AccordionValue> | null = null;

function register(): void {
  registration?.unregister();
  registration = context.registerTrigger({
    disabled: collapsible.disabled,
    element,
    value: item.value.value,
  });
}

watch(() => item.value.value, register, { flush: "sync", immediate: true });
onUnmounted(() => {
  registration?.unregister();
  registration = null;
});

function onClick(event: MouseEvent): void {
  if (collapsible.disabled.value) {
    event.preventDefault();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) collapsible.toggle(event);
}

function onKeydown(event: KeyboardEvent): void {
  context.handleTriggerKeydown(item.value.value, event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type AccordionTriggerSetupExpose = Omit<AccordionTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies AccordionTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="collapsible.triggerId.value"
    ref="element"
    type="button"
    :disabled="collapsible.disabled.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-expanded="collapsible.open.value ? 'true' : 'false'"
    :aria-controls="collapsible.contentId.value"
    :aria-disabled="item.locked.value ? 'true' : undefined"
    data-vize-ui="accordion-trigger"
    part="trigger"
    :data-state="collapsible.state.value"
    :data-orientation="context.orientation.value"
    :data-disabled="collapsible.disabled.value ? 'true' : undefined"
    :data-locked="item.locked.value ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot
      :disabled="collapsible.disabled.value"
      :locked="item.locked.value"
      :open="collapsible.open.value"
      :state="collapsible.state.value"
      :value="item.value.value"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
