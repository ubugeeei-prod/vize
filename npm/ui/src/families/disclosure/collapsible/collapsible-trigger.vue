<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { collapsibleContext } from "./collapsible-context.ts";
import type { CollapsibleSlotState, CollapsibleTriggerExpose } from "./collapsible-types.ts";

const {
  type = "button",
  disabled = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Remove this trigger from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

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
  /** Trigger contents. Receives the current Collapsible state and trigger availability. */
  default(props: CollapsibleSlotState): unknown;
}>();

const context = collapsibleContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const triggerDisabled = computed(() => context.disabled.value || disabled);

function onClick(event: MouseEvent): void {
  if (triggerDisabled.value) {
    event.preventDefault();
    return;
  }
  emit("click", event);
  if (!event.defaultPrevented) context.toggle(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type CollapsibleTriggerSetupExpose = Omit<CollapsibleTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies CollapsibleTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    :type
    :disabled="triggerDisabled"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.contentId.value"
    data-vize-ui="collapsible-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-disabled="triggerDisabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot :disabled="triggerDisabled" :open="context.open.value" :state="context.state.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
