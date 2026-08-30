<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watchEffect } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../../deterministic-id.ts";
import { radioGroupContext } from "./radio-group-context.ts";
import type { RadioGroupItemExpose, RadioGroupItemState } from "./radio-group-types.ts";

const {
  id = undefined,
  value = "on",
  disabled = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned radio id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native radio value submitted when this item is selected.
   *
   * @default "on"
   */
  readonly value?: string;

  /**
   * Disable this item while preserving the rest of the group.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no associated label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this radio.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe this radio.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const context = radioGroupContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "radio" });
const checked = computed(() => context.value.value === value);
const itemDisabled = computed(() => context.disabled.value || disabled);
const itemState = computed<RadioGroupItemState>(() => {
  if (itemDisabled.value) return "disabled";
  return checked.value ? "checked" : "unchecked";
});

function syncNativeState(): void {
  if (element.value === null) return;
  element.value.checked = checked.value;
}

watchEffect(syncNativeState);

function onChange(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  if (event.currentTarget.checked) context.selectValue(value, event);
  void nextTick(syncNativeState);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type RadioGroupItemSetupExpose = Omit<RadioGroupItemExpose, "checked" | "disabled" | "element"> & {
  readonly checked: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = {
  checked,
  disabled: itemDisabled,
  element,
  focus,
  value,
} satisfies RadioGroupItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <input
    :id="controlId"
    ref="element"
    type="radio"
    :name="context.name.value"
    :value
    :checked
    :disabled="itemDisabled"
    :required="context.required.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="radio-group-item"
    part="item"
    :data-state="itemState"
    :data-checked="checked ? 'true' : 'false'"
    :data-disabled="itemDisabled ? 'true' : undefined"
    :data-required="context.required.value ? 'true' : undefined"
    :data-invalid="context.invalid.value ? 'true' : undefined"
    @change="onChange"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
