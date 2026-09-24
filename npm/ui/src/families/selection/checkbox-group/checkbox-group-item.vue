<script setup lang="ts" generic="Value">
import { computed, useTemplateRef, watchEffect } from "vue";

import { checkboxGroupContext } from "./checkbox-group-context.ts";

const {
  value,
  id = undefined,
  disabled = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Option this checkbox toggles; must be one of the group's `options` (by identity).
   *
   * @default required
   */
  readonly value: Value;

  /**
   * Native id, typically referenced by a `<label for>`.
   *
   * @default undefined
   */
  readonly id?: string;

  /**
   * Disable this checkbox in addition to group and `isOptionDisabled` rules.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no wrapping or associated label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label this checkbox.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired after the user toggles this checkbox, with its new checked state and native `Event`. */
  change: [checked: boolean, nativeEvent: Event];
}>();

const context = checkboxGroupContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const index = computed(() => context.indexOf(value));
const checked = computed(() => context.isIndexSelected(index.value));
const itemDisabled = computed(() => disabled || context.isIndexDisabled(index.value));
// Native "at least one" validation: every box is required only while nothing is selected.
const nativeRequired = computed(() => context.required.value && !context.hasSelection.value);

function syncNativeState(): void {
  if (element.value !== null && element.value.checked !== checked.value) {
    element.value.checked = checked.value;
  }
}

watchEffect(syncNativeState, { flush: "post" });

function onChange(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  const next = event.currentTarget.checked;
  if (!itemDisabled.value) context.toggleIndex(index.value, next);
  emit("change", next, event);
  // Controlled parents may reject the change; restore the rendered state.
  queueMicrotask(syncNativeState);
}
</script>

<template>
  <input
    :id
    ref="element"
    type="checkbox"
    :name="context.name.value"
    :form="context.form.value"
    :value="context.formValueOf(index)"
    :checked
    :disabled="itemDisabled"
    :required="nativeRequired"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-invalid="context.ariaInvalid.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    part="item"
    data-vize-ui="checkbox-group-item"
    :data-index="index"
    :data-state="checked ? 'checked' : 'unchecked'"
    :data-disabled="itemDisabled ? 'true' : undefined"
    @change="onChange"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
