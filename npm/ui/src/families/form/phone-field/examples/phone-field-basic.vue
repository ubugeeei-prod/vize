<!-- Phone number field bound with v-model to an E.164 value, with a country select and pattern formatting. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  PhoneField,
  PhoneFieldCountrySelect,
  PhoneFieldInput,
  definePhoneCountries,
} from "../phone-field.ts";

const countries = definePhoneCountries([
  { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999", trunkPrefix: "0" },
  { code: "US", name: "United States", dialCode: "1", pattern: "(999) 999-9999" },
  { code: "GB", name: "United Kingdom", dialCode: "44", pattern: "9999 999999", trunkPrefix: "0" },
]);

const phone = ref("");
const inputId = useId();
</script>

<template>
  <div>
    <label :for="inputId">Mobile number</label>
    <PhoneField
      :id="inputId"
      v-slot="{ international }"
      v-model="phone"
      :countries
      default-country="JP"
      name="phone"
      required
    >
      <PhoneFieldCountrySelect />
      <PhoneFieldInput autocomplete="tel-national" />
      <output>{{ international }}</output>
    </PhoneField>
  </div>
</template>
