<script setup lang="ts">
import { useTemplateRef, watchEffect } from "vue";

import { checkboxGroupContext } from "./checkbox-group-context.ts";

const {
  id = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Native id, typically referenced by a `<label for>`.
   *
   * @default undefined
   */
  readonly id?: string;

  /**
   * Accessible name when no wrapping or associated label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the parent checkbox.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired after the user toggles every enabled option, with the requested state and native `Event`. */
  change: [selected: boolean, nativeEvent: Event];
}>();

const context = checkboxGroupContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const state = context.selectAllState;

function syncNativeState(): void {
  if (element.value === null) return;
  element.value.checked = state.value === "checked";
  element.value.indeterminate = state.value === "indeterminate";
}

watchEffect(syncNativeState, { flush: "post" });

function onChange(event: Event): void {
  // Mixed and unchecked both select everything, matching the APG tri-state checkbox.
  const selected = state.value !== "checked";
  context.toggleAll(selected);
  emit("change", selected, event);
  queueMicrotask(syncNativeState);
}
</script>

<template>
  <input
    :id
    ref="element"
    type="checkbox"
    :checked="state === 'checked'"
    :disabled="context.selectAllDisabled.value"
    :aria-checked="state === 'indeterminate' ? 'mixed' : state === 'checked' ? 'true' : 'false'"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    part="select-all"
    data-vize-ui="checkbox-group-select-all"
    :data-state="state"
    @change="onChange"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
