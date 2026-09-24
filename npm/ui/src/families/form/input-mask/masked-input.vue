<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useInputMask } from "./input-mask-runtime.ts";
import type {
  MaskedInputEmits,
  MaskedInputExpose,
  MaskedInputProps,
  MaskedInputState,
} from "./input-mask-types.ts";

const {
  mask,
  tokens = undefined,
  placeholderChar = "_",
  lazy = true,
  eager = false,
  valueFormat = "raw",
  modelValue = undefined,
  defaultValue = "",
  id = undefined,
  name = undefined,
  disabled = false,
  readOnly = false,
  required = false,
  placeholder = undefined,
  autocomplete = undefined,
  inputMode = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<MaskedInputProps>();

const emit = defineEmits<MaskedInputEmits>();

const element = useTemplateRef<HTMLInputElement>("element");
const controlId = useDeterministicId({ id: () => id, hint: "masked-input" });
const controller = useInputMask({
  mask: () => mask,
  tokens: () => tokens,
  placeholderChar: () => placeholderChar,
  lazy: () => lazy,
  eager: () => eager,
  valueFormat: () => valueFormat,
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (value) => emit("update:modelValue", value),
  onComplete: (result) => emit("complete", result),
});
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed<MaskedInputState>(() => {
  if (disabled) return "disabled";
  if (readOnly) return "readonly";
  if (controller.raw.value.length === 0) return "empty";
  return controller.complete.value ? "complete" : "incomplete";
});

function onInput(event: Event): void {
  controller.handleInput(event);
}

watch(
  element,
  (input, _previous, onCleanup) => {
    const form = input?.form;
    if (form === undefined || form === null) return;
    const onReset = () => {
      // Native reset restores the `value` attribute first; re-apply our state after it.
      queueMicrotask(() => {
        if (modelValue === undefined) controller.reset();
        if (input !== null) input.value = controller.masked.value;
      });
    };
    form.addEventListener("reset", onReset);
    onCleanup(() => form.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

type MaskedInputSetupExpose = Omit<MaskedInputExpose, "complete" | "element" | "masked" | "raw"> & {
  readonly complete: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly masked: ComputedRef<string>;
  readonly raw: ComputedRef<string>;
};

const exposed = {
  complete: controller.complete,
  element,
  masked: controller.masked,
  raw: controller.raw,
  focus: (options?: FocusOptions) => element.value?.focus(options),
  setValue: controller.setValue,
  reset: controller.reset,
} satisfies MaskedInputSetupExpose;

defineExpose(exposed);
</script>

<template>
  <input
    :id="controlId"
    ref="element"
    type="text"
    :name
    :value="controller.masked.value"
    :disabled
    :readonly="readOnly"
    :required
    :placeholder
    :autocomplete
    :inputmode="inputMode ?? controller.inputMode.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-errormessage="ariaInvalidValue === undefined ? undefined : ariaErrormessage"
    :aria-invalid="ariaInvalidValue"
    part="input"
    data-vize-ui="masked-input"
    :data-state="dataState"
    :data-complete="controller.complete.value ? 'true' : 'false'"
    @input="onInput"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
