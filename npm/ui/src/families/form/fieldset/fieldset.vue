<script setup lang="ts">
import { computed, toRef, useTemplateRef, watch } from "vue";

import { fieldsetContext } from "./fieldset-context.ts";
import { useFieldWiring } from "../field-wiring/field-wiring-runtime.ts";
import type { FormFieldError } from "../form/form-types.ts";
import type { FieldsetSlotState, FieldsetState } from "./fieldset-types.ts";

const {
  id = undefined,
  name = undefined,
  form = undefined,
  errors = [],
  invalid = undefined,
  disabled = false,
  hasDescription = false,
  hasErrorMessage = true,
} = defineProps<{
  /**
   * Fieldset id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Group name matched against `errors[].name` (for example `"address"`), also
   * set as the native fieldset `name`.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of a form owner outside the component tree.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Normalized form errors; entries whose `name` equals `name` mark the group invalid.
   *
   * @default []
   */
  readonly errors?: readonly FormFieldError[];

  /**
   * Consumer-owned invalid override. `undefined` derives it from matching errors.
   *
   * @default undefined
   */
  readonly invalid?: boolean;

  /**
   * Natively disable the fieldset and every descendant form control.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Whether a FieldsetDescription is rendered and joins `aria-describedby`.
   *
   * @default false
   */
  readonly hasDescription?: boolean;

  /**
   * Whether a FieldsetErrorMessage is rendered while invalid and joins `aria-describedby`.
   *
   * @default true
   */
  readonly hasErrorMessage?: boolean;
}>();

const emit = defineEmits<{
  /** Fired after the derived invalid state changes, with the matching errors. */
  "invalid-change": [invalid: boolean, errors: readonly FormFieldError[]];
}>();

defineSlots<{
  /** Renders the legend, description, fields, and error message with group state. */
  default(props: FieldsetSlotState): unknown;
}>();

const element = useTemplateRef<HTMLFieldSetElement>("element");
const matchingErrors = computed(() =>
  name === undefined ? [] : errors.filter((error) => error.name === name),
);
const isInvalid = computed(() => invalid ?? matchingErrors.value.length > 0);
const errorMessage = computed(() => matchingErrors.value[0]?.message);
// Field wiring supplies the deterministic ids; the fieldset itself is named by its legend.
const wiring = useFieldWiring({
  id: toRef(() => id),
  invalid: isInvalid,
  hasDescription: toRef(() => hasDescription),
  hasErrorMessage: toRef(() => hasErrorMessage),
});
const describedBy = computed(() => wiring.fieldProps.value["aria-describedby"]);
const dataState = computed<FieldsetState>(() => {
  if (disabled) return "disabled";
  return isInvalid.value ? "invalid" : "valid";
});
const slotState = computed<FieldsetSlotState>(() => ({
  id: wiring.fieldId.value,
  invalid: isInvalid.value,
  disabled,
  errors: matchingErrors.value,
  errorMessage: errorMessage.value,
  state: dataState.value,
}));

watch(isInvalid, (next) => emit("invalid-change", next, matchingErrors.value));

fieldsetContext.provide({
  descriptionId: wiring.descriptionId,
  errorMessageId: wiring.errorMessageId,
  invalid: isInvalid,
  disabled: computed(() => disabled),
  errors: matchingErrors,
  errorMessage,
});

defineExpose({ element, invalid: isInvalid, id: wiring.fieldId });
</script>

<template>
  <fieldset
    :id="wiring.fieldId.value"
    ref="element"
    :name
    :form
    :disabled
    :aria-describedby="describedBy"
    part="root"
    data-vize-ui="fieldset"
    :data-state="dataState"
    :data-invalid="isInvalid ? 'true' : 'false'"
  >
    <slot v-bind="slotState" />
  </fieldset>
</template>

<style scoped>
/* Headless by design. Native fieldset styling remains consumer-owned. */
</style>
