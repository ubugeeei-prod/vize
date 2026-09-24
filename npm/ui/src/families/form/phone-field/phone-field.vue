<script setup lang="ts" generic="Code extends string">
import { computed, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { phoneFieldContext } from "./phone-field-context.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  formatNationalNumber,
  parsePhoneNumber,
  phoneDigitLimit,
  toE164,
} from "./phone-field-country.ts";
import type { PhoneCountry } from "./phone-field-country.ts";
import type {
  PhoneFieldAriaInvalid,
  PhoneFieldExpose,
  PhoneFieldSlotState,
  PhoneFieldState,
} from "./phone-field-types.ts";
import { createInputMask } from "../input-mask/input-mask.ts";

const {
  countries,
  modelValue = undefined,
  defaultValue = "",
  country = undefined,
  defaultCountry = undefined,
  id = undefined,
  name = undefined,
  disabled = false,
  required = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  ariaErrormessage = undefined,
  ariaInvalid = false,
} = defineProps<{
  /**
   * Supported countries; their `code` literals type `v-model:country`.
   *
   * @default required
   */
  readonly countries: readonly PhoneCountry<Code>[];

  /**
   * Controlled E.164 value (`+819012345678`); `""` is empty. `undefined` selects uncontrolled use.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial uncontrolled value (E.164 or national text), also restored by form reset.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Controlled country code (`v-model:country`).
   *
   * @default undefined
   */
  readonly country?: Code;

  /**
   * Initial uncontrolled country.
   *
   * @default countries[0].code
   */
  readonly defaultCountry?: Code;

  /**
   * Id of the number input. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name; a hidden input submits the E.164 value.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Disable the input, the country select, and form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark the number as required for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Accessible name of the number input.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the number input.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Ids that describe the number input.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Id of the validation error message used while invalid.
   *
   * @default undefined
   */
  readonly ariaErrormessage?: string;

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: PhoneFieldAriaInvalid;
}>();

const emit = defineEmits<{
  /** Fired when the E.164 value requests a change. */
  "update:modelValue": [value: string];

  /** Fired when the country requests a change (`v-model:country`). */
  "update:country": [code: Code];

  /** Fired when the national number fills the country pattern. */
  complete: [e164: string, country: PhoneCountry<Code>];
}>();

defineSlots<{
  /** Renders the country select, number input, and any hints with typed phone state. */
  default(props: PhoneFieldSlotState<Code>): unknown;
}>();

const root = useTemplateRef<HTMLDivElement>("root");
const inputId = useDeterministicId({ id: () => id, hint: "phone" });

const firstCountry = computed<PhoneCountry<Code>>(() => {
  const first = countries[0];
  if (first === undefined) {
    throw new TypeError("VIZE_UI_PHONE_FIELD_COUNTRIES: countries must not be empty");
  }
  return first;
});

function findCountry(code: string | undefined): PhoneCountry<Code> | undefined {
  return countries.find((candidate) => candidate.code === code);
}

const countryState = useControllableState<Code>({
  value: () =>
    country === undefined ? undefined : (findCountry(country)?.code ?? firstCountry.value.code),
  defaultValue: () => findCountry(defaultCountry)?.code ?? firstCountry.value.code,
  onChange: (code) => emit("update:country", code),
});
const selectedCountry = computed(() => findCountry(countryState.value.value) ?? firstCountry.value);

function normalizeStored(text: string): string {
  const parsed = parsePhoneNumber(text, countries, selectedCountry.value);
  return parsed === undefined ? "" : toE164(parsed.nationalNumber, parsed.country);
}

const valueState = useControllableState<string>({
  value: () => (modelValue === undefined ? undefined : modelValue),
  defaultValue: () => normalizeStored(defaultValue),
  onChange: (value) => emit("update:modelValue", value),
});
const parsed = computed(() =>
  valueState.value.value.length === 0
    ? undefined
    : parsePhoneNumber(valueState.value.value, countries, selectedCountry.value),
);
// A stored number from another country wins over the selected country.
const activeCountry = computed(() => parsed.value?.country ?? selectedCountry.value);
const nationalNumber = computed(() => parsed.value?.nationalNumber ?? "");
const e164 = computed(() => toE164(nationalNumber.value, activeCountry.value));
const formatted = computed(() => formatNationalNumber(nationalNumber.value, activeCountry.value));
const complete = computed(
  () =>
    nationalNumber.value.length > 0 &&
    activeCountry.value.pattern !== undefined &&
    createInputMask(activeCountry.value.pattern).fromRaw(nationalNumber.value).complete,
);
const ariaInvalidValue = computed(() => {
  if (ariaInvalid === false) return undefined;
  return ariaInvalid === true ? "true" : ariaInvalid;
});
const dataState = computed<PhoneFieldState>(() => {
  if (disabled) return "disabled";
  if (nationalNumber.value.length === 0) return "empty";
  return complete.value ? "complete" : "incomplete";
});

watch(complete, (next) => {
  if (next) emit("complete", e164.value, activeCountry.value);
});

function setNationalNumber(digits: string): void {
  if (disabled) return;
  const limited = digits.slice(0, phoneDigitLimit(activeCountry.value));
  valueState.set(toE164(limited, activeCountry.value));
}

function setCountry(code: Code): boolean {
  const next = findCountry(code);
  if (next === undefined || disabled) return false;
  const e164Next = toE164(nationalNumber.value.slice(0, phoneDigitLimit(next)), next);
  const changed = countryState.set(next.code);
  if (e164Next.length > 0) valueState.set(e164Next);
  return changed || next !== activeCountry.value;
}

function setText(text: string): boolean {
  if (disabled) return false;
  const result = parsePhoneNumber(text, countries, activeCountry.value);
  if (result === undefined) return false;
  if (result.country !== selectedCountry.value) countryState.set(result.country.code);
  valueState.set(toE164(result.nationalNumber, result.country));
  return true;
}

watch(
  root,
  (element, _previous, onCleanup) => {
    const owner = element?.closest("form");
    if (owner === null || owner === undefined) return;
    const onReset = () => {
      if (!valueState.controlled.value) valueState.reset();
      if (!countryState.controlled.value) countryState.reset();
    };
    owner.addEventListener("reset", onReset);
    onCleanup(() => owner.removeEventListener("reset", onReset));
  },
  { flush: "post", immediate: true },
);

phoneFieldContext.provide({
  inputId,
  countries: computed(() => countries),
  country: activeCountry,
  nationalNumber,
  disabled: computed(() => disabled),
  required: computed(() => required),
  ariaLabel: computed(() => ariaLabel),
  ariaLabelledby: computed(() => ariaLabelledby),
  ariaDescribedby: computed(() => ariaDescribedby),
  ariaErrormessage: computed(() => ariaErrormessage),
  ariaInvalid: ariaInvalidValue,
  setCountryCode: (code: string) => {
    const next = findCountry(code);
    return next === undefined ? false : setCountry(next.code);
  },
  setNationalNumber,
  setText,
});

const slotState = computed<PhoneFieldSlotState<Code>>(() => ({
  country: activeCountry.value,
  nationalNumber: nationalNumber.value,
  e164: e164.value,
  formatted: formatted.value,
  international:
    nationalNumber.value.length === 0 ? "" : `+${activeCountry.value.dialCode} ${formatted.value}`,
  complete: complete.value,
  disabled,
  state: dataState.value,
}));

type PhoneFieldSetupExpose = Omit<PhoneFieldExpose<Code>, keyof PhoneFieldSlotState<Code>> & {
  readonly [Key in keyof PhoneFieldSlotState<Code>]: ComputedRef<PhoneFieldSlotState<Code>[Key]>;
} & { readonly root: typeof root };

function field<Key extends keyof PhoneFieldSlotState<Code>>(
  key: Key,
): ComputedRef<PhoneFieldSlotState<Code>[Key]> {
  return computed(() => slotState.value[key]);
}

const exposed = {
  country: field("country"),
  nationalNumber: field("nationalNumber"),
  e164: field("e164"),
  formatted: field("formatted"),
  international: field("international"),
  complete: field("complete"),
  disabled: field("disabled"),
  state: field("state"),
  root,
  setCountry,
  setValue: setText,
  clear: () => {
    valueState.set("");
  },
} satisfies PhoneFieldSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="root"
    role="group"
    part="root"
    data-vize-ui="phone-field"
    :data-state="dataState"
    :data-country="activeCountry.code"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <input v-if="name !== undefined" type="hidden" :name :value="e164" :disabled />
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
