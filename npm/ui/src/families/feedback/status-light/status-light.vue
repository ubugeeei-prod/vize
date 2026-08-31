<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  StatusLightAriaState,
  StatusLightElement,
  StatusLightExpose,
  StatusLightProps,
  StatusLightRole,
  StatusLightSize,
  StatusLightSlotState,
  StatusLightState,
  StatusLightTone,
} from "./status-light-types.ts";

const {
  as = "span",
  state = "unknown",
  tone = "neutral",
  size = "md",
  role = "img",
  atomic = true,
  ariaHidden = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<StatusLightProps>();

defineSlots<{
  /** Optional visual indicator contents. Receives current state for composition. */
  default(props: StatusLightSlotState): unknown;
}>();

const element = useTemplateRef<StatusLightElement>("element");
const stateValue = computed(() => state);
const toneValue = computed(() => tone);
const sizeValue = computed(() => size);
const roleValue = computed(() => role);
const hasAccessibleName = computed(() => hasText(ariaLabel) || hasText(ariaLabelledby));
const decorative = computed(() => ariaHidden ?? !hasAccessibleName.value);
const ariaState = computed<StatusLightAriaState>(() =>
  decorative.value ? "decorative" : roleValue.value,
);
const slotState = computed<StatusLightSlotState>(() => ({
  ariaState: ariaState.value,
  decorative: decorative.value,
  size: sizeValue.value,
  state: stateValue.value,
  tone: toneValue.value,
}));

type StatusLightSetupExpose = Omit<
  StatusLightExpose,
  "ariaState" | "decorative" | "element" | "size" | "state" | "tone"
> & {
  readonly ariaState: typeof ariaState;
  readonly decorative: typeof decorative;
  readonly element: typeof element;
  readonly size: typeof sizeValue;
  readonly state: typeof stateValue;
  readonly tone: typeof toneValue;
};

const exposed = {
  ariaState,
  decorative,
  element,
  size: sizeValue,
  state: stateValue,
  tone: toneValue,
} satisfies StatusLightSetupExpose;

defineExpose(exposed);

function hasText(value: string | undefined): boolean {
  return value != null && value.trim().length > 0;
}
</script>

<template>
  <component
    :is="as"
    ref="element"
    :role="ariaState === 'decorative' ? undefined : roleValue"
    :aria-hidden="ariaState === 'decorative' ? 'true' : undefined"
    :aria-label="ariaState === 'decorative' ? undefined : ariaLabel"
    :aria-labelledby="ariaState === 'decorative' ? undefined : ariaLabelledby"
    :aria-describedby="ariaState === 'decorative' ? undefined : ariaDescribedby"
    :aria-live="ariaState === 'status' ? 'polite' : undefined"
    :aria-atomic="ariaState === 'status' ? (atomic ? 'true' : 'false') : undefined"
    data-vize-ui="status-light"
    part="root"
    :data-state="stateValue"
    :data-tone="toneValue"
    :data-size="sizeValue"
    :data-aria-state="ariaState"
    :data-decorative="decorative ? 'true' : 'false'"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
