<script setup lang="ts">
import { computed } from "vue";

import { phoneFieldContext } from "./phone-field-context.ts";
import { useInputMask } from "../input-mask/input-mask.ts";

const { placeholder = undefined, autocomplete = "tel-national" } = defineProps<{
  /**
   * Native placeholder; defaults to nothing so consumers can show a sample number.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Autocomplete token: `tel-national` beside a country select, `tel` when international text is expected.
   *
   * @default "tel-national"
   */
  readonly autocomplete?: string;
}>();

const context = phoneFieldContext.use();
const mask = useInputMask({
  mask: () => context.country.value.pattern ?? "9".repeat(15),
  value: () => context.nationalNumber.value,
  valueFormat: "raw",
  onChange: (digits) => context.setNationalNumber(digits),
});
const hint = computed(() => `+${context.country.value.dialCode}`);

function onInput(event: Event): void {
  if (!(event.target instanceof HTMLInputElement)) return;
  const text = event.target.value;
  // International text ("+44 20…", "0044…") re-targets the country instead of masking.
  if (/^\s*(?:\+|00)/.test(text) && context.setText(text)) {
    event.target.value = mask.masked.value;
    return;
  }
  mask.handleInput(event);
}

function onPaste(event: ClipboardEvent): void {
  const text = event.clipboardData?.getData("text") ?? "";
  if (!/^\s*(?:\+|00)/.test(text)) return;
  event.preventDefault();
  context.setText(text);
}
</script>

<template>
  <input
    :id="context.inputId.value"
    type="tel"
    inputmode="tel"
    :value="mask.masked.value"
    :autocomplete
    :placeholder
    :disabled="context.disabled.value"
    :required="context.required.value"
    :aria-label="context.ariaLabel.value"
    :aria-labelledby="context.ariaLabelledby.value"
    :aria-describedby="context.ariaDescribedby.value"
    :aria-errormessage="
      context.ariaInvalid.value === undefined ? undefined : context.ariaErrormessage.value
    "
    :aria-invalid="context.ariaInvalid.value"
    part="input"
    data-vize-ui="phone-field-input"
    :data-dial-code="hint"
    @input="onInput"
    @paste="onPaste"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
