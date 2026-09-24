<script setup lang="ts">
import { computed } from "vue";

import { phoneFieldContext } from "./phone-field-context.ts";
import type { PhoneCountry } from "./phone-field-country.ts";
import NativeSelect from "../../selection/native-select/native-select.vue";

const { ariaLabel = "Country", getOptionLabel = undefined } = defineProps<{
  /**
   * Accessible name of the country select.
   *
   * @default "Country"
   */
  readonly ariaLabel?: string;

  /**
   * Option text for a country.
   *
   * @default (country) => `${country.name} (+${country.dialCode})`
   */
  readonly getOptionLabel?: (country: PhoneCountry) => string;
}>();

const context = phoneFieldContext.use();
const options = computed(() =>
  context.countries.value.map((country) => ({
    value: country.code,
    label: (getOptionLabel ?? ((item: PhoneCountry) => `${item.name} (+${item.dialCode})`))(
      country,
    ),
  })),
);

function onUpdate(value: string | readonly string[]): void {
  if (typeof value === "string") context.setCountryCode(value);
}
</script>

<template>
  <NativeSelect
    :model-value="context.country.value.code"
    :options
    :disabled="context.disabled.value"
    :aria-label
    :aria-controls="context.inputId.value"
    part="country-select"
    data-vize-ui="phone-field-country-select"
    @update:model-value="onUpdate"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
