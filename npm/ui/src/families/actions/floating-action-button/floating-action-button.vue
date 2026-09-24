<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  FloatingActionButtonExpose,
  FloatingActionButtonPlacement,
  FloatingActionButtonSlotState,
} from "./floating-action-button-types.ts";

const {
  placement = "bottom-end",
  extended = false,
  disabled = false,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Viewport corner or edge the consumer anchors the button to, mirrored to `data-placement`.
   *
   * @default "bottom-end"
   */
  readonly placement?: FloatingActionButtonPlacement;

  /**
   * Whether the button shows a visible text label next to its icon.
   *
   * @default false
   */
  readonly extended?: boolean;

  /**
   * Remove the button from activation and sequential focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name. Required in practice for icon-only buttons.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired when the button is activated. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Icon and optional label. Receives placement and extended state. */
  default(props: FloatingActionButtonSlotState): unknown;
}>();

const element = useTemplateRef<HTMLButtonElement>("element");
const slotState = computed<FloatingActionButtonSlotState>(() => ({ extended, placement }));

function onClick(event: MouseEvent): void {
  emit("click", event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type FloatingActionButtonSetupExpose = Omit<FloatingActionButtonExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies FloatingActionButtonSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    data-vize-ui="floating-action-button"
    part="root"
    :data-placement="placement"
    :data-extended="extended ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
