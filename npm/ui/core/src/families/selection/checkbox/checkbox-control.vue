<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch, watchEffect } from "vue";

import { getCheckboxState } from "./checkbox-state.ts";
import { useControllableState } from "../../../controllable-state.ts";

const {
  modelValue = undefined,
  defaultChecked = false,
  indeterminate = false,
  disabled = false,
  ariaLabel,
} = defineProps<{
  /**
   * Controlled checked value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: boolean;

  /**
   * Initial unchecked or checked state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultChecked?: boolean;

  /**
   * Render and announce a mixed checked state.
   *
   * @default false
   */
  readonly indeterminate?: boolean;

  /**
   * Disable interaction and native form submission.
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
}>();

const emit = defineEmits<{
  /** Fired when the checked state requests a new controlled boolean value. */
  "update:modelValue": [value: boolean];
  /** Fired when user interaction emits the next mixed-state boolean. */
  "update:indeterminate": [value: boolean];
  /** Fired after the native checkbox change event is processed with the next checked boolean and native `Event`. */
  change: [value: boolean, nativeEvent: Event];
}>();

const element = useTemplateRef<HTMLInputElement>("element");
const state = useControllableState({
  value: () => modelValue,
  defaultValue: () => defaultChecked,
  onChange: (value) => emit("update:modelValue", value),
});
const checked = state.value;
const visualState = computed(() => getCheckboxState(checked.value, indeterminate));

function syncNativeState(): void {
  if (element.value === null) return;
  element.value.checked = checked.value;
  element.value.indeterminate = indeterminate;
}

watchEffect(syncNativeState);

watch(
  element,
  (input, _previous, onCleanup) => {
    const form = input?.form;
    if (form === undefined || form === null) return;
    const onReset = () => {
      if (!state.controlled.value) state.reset();
      void nextTick(syncNativeState);
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

function onChange(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  const next = event.currentTarget.checked;
  state.set(next);
  if (indeterminate) emit("update:indeterminate", false);
  emit("change", next, event);
  void nextTick(syncNativeState);
}

/** Move focus to the native checkbox. */
function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

defineExpose({ element, checked, focus, reset: state.reset, setChecked: state.set });
</script>

<template>
  <input
    ref="element"
    type="checkbox"
    :checked
    :disabled
    :aria-label="ariaLabel"
    :aria-checked="indeterminate ? 'mixed' : checked"
    data-vize-ui="checkbox"
    :data-state="visualState"
    @change="onChange"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
