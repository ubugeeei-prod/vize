<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { selectContext } from "./select-context.ts";
import type { SelectTriggerExpose } from "./select-types.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
} = defineProps<{
  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids of visible labels for the select.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the select.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Id of the validation message announced while invalid.
   *
   * @default undefined
   */
  readonly ariaErrormessage?: string;
}>();

defineSlots<{
  /** Trigger content, usually `SelectValue` plus an icon. Receives open and selection state. */
  default(props: {
    readonly open: boolean;
    readonly disabled: boolean;
    readonly empty: boolean;
  }): unknown;
}>();

const context = selectContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const slotState = computed(() => ({
  disabled: context.disabled.value,
  empty: context.selected.value.length === 0,
  open: context.open.value,
}));
const interactiveProps = computed<{
  readonly role: "combobox";
  readonly onClick: (event: MouseEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({
  role: "combobox",
  onClick,
  onKeydown: (event: KeyboardEvent) => context.onTriggerKeydown(event),
}));

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (context.disabled.value) return;
  context.setOpen(!context.open.value, event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type SelectTriggerSetupExpose = Omit<SelectTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies SelectTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    v-bind="interactiveProps"
    type="button"
    :disabled="context.disabled.value"
    aria-haspopup="listbox"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.open.value ? context.listboxId.value : undefined"
    :aria-activedescendant="context.activeDescendant.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-required="context.required.value ? 'true' : undefined"
    :aria-invalid="context.invalid.value ? 'true' : undefined"
    :aria-errormessage="context.invalid.value ? ariaErrormessage : undefined"
    data-vize-ui="select-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    :data-placeholder="context.selected.value.length === 0 ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
