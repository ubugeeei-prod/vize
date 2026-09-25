<!-- Paged year picker bound with v-model and a fixed SSR-safe today. -->
<script setup lang="ts">
import { ref } from "vue";

import { YearPicker } from "../year-picker.ts";

const vintage = ref<number | null>(2020);
</script>

<template>
  <div>
    <YearPicker
      v-model="vintage"
      name="vintage"
      locale="en-US"
      aria-label="Vintage year"
      :today="{ year: 2026, month: 10, day: 1 }"
      :min="1990"
    />
    <output>{{ vintage ?? "No year" }}</output>
  </div>
</template>
