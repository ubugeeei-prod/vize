<!-- Locale provider that switches language and direction for a subtree and formats a price in it. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { LocaleProvider, resolveListFormatter, resolveNumberFormatter } from "../locale.ts";

const locale = ref("en-US");
const selectId = useId();
const locales = [
  { value: "en-US", label: "English (United States)" },
  { value: "de-DE", label: "Deutsch (Deutschland)" },
  { value: "ar-EG", label: "العربية (مصر)" },
];

function formatPrice(tag: string): string {
  return resolveNumberFormatter(tag, { style: "currency", currency: "EUR" }).format(1299.5);
}
</script>

<template>
  <div>
    <label :for="selectId">Language</label>
    <select :id="selectId" v-model="locale">
      <option v-for="item in locales" :key="item.value" :value="item.value">
        {{ item.label }}
      </option>
    </select>
    <LocaleProvider v-slot="{ locale: active, direction }" :locale direction="auto">
      <p>Price: {{ formatPrice(active) }}</p>
      <p>
        Direction: {{ direction }}; ships in
        {{ resolveListFormatter(active).format(["S", "M", "L"]) }}
      </p>
    </LocaleProvider>
  </div>
</template>
