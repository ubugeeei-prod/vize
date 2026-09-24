<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";

import { comboboxContext } from "./combobox-context.ts";

const {
  placeholder = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  autocomplete = "off",
  spellcheck = false,
} = defineProps<{
  /**
   * Hint text shown while the input is empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids of visible labels; also labels the listbox.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the combobox.
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

  /**
   * Native browser autofill hint; `off` keeps browser suggestions out of the listbox's way.
   *
   * @default "off"
   */
  readonly autocomplete?: string;

  /**
   * Native spellchecking.
   *
   * @default false
   */
  readonly spellcheck?: boolean;
}>();

const context = comboboxContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const handlers = computed(() => ({
  onBlur: (event: FocusEvent) => context.onBlur(event),
  onFocus: (event: FocusEvent) => context.onFocus(event),
  onInput: (event: Event) => context.onInput(event),
  onKeydown: (event: KeyboardEvent) => context.onKeydown(event),
}));

watch(
  () => ariaLabelledby,
  (value) => {
    context.labelledby.value = value;
  },
  { immediate: true },
);

onMounted(() => {
  context.inputElement.value = element.value;
});

onUnmounted(() => {
  if (context.inputElement.value === element.value) context.inputElement.value = null;
});

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

defineExpose({ element, focus });
</script>

<template>
  <input
    :id="context.inputId.value"
    ref="element"
    v-bind="handlers"
    type="text"
    role="combobox"
    :value="context.inputValue.value"
    :placeholder
    :autocomplete
    :spellcheck="spellcheck ? 'true' : 'false'"
    autocapitalize="off"
    :disabled="context.disabled.value"
    :readonly="context.readonly.value"
    :required="context.nativeRequired.value"
    :aria-autocomplete="context.autocomplete.value"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.open.value ? context.listboxId.value : undefined"
    :aria-activedescendant="context.activeDescendant.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-required="context.required.value ? 'true' : undefined"
    :aria-invalid="context.invalid.value ? 'true' : undefined"
    :aria-errormessage="context.invalid.value ? ariaErrormessage : undefined"
    data-vize-ui="combobox-input"
    part="input"
    :data-state="context.open.value ? 'open' : 'closed'"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
