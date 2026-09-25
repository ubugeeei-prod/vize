<!-- Month picker bound with v-model, a fixed SSR-safe today, and a selectable range. -->
<script setup lang="ts">
import { ref } from "vue";

import { formatIsoYearMonth, MonthPicker } from "../month-picker.ts";
import type { PlainYearMonth } from "../month-picker.ts";

const billing = ref<PlainYearMonth | null>({ year: 2026, month: 10 });
</script>

<template>
  <div>
    <MonthPicker
      v-model="billing"
      name="billing-month"
      locale="en-US"
      aria-label="Billing month"
      :today="{ year: 2026, month: 10, day: 1 }"
      :min="{ year: 2025, month: 1 }"
      :max="{ year: 2027, month: 12 }"
    />
    <output>{{ billing ? formatIsoYearMonth(billing) : "No month" }}</output>
  </div>
</template>
