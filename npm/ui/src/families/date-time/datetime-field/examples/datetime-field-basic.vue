<!-- Controlled date-time field in 24-hour format with a lower bound and ISO output. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { createPlainDateTime, DateTimeField, formatIsoDateTime } from "../datetime-field.ts";
import type { PlainDateTime } from "../datetime-field.ts";

const departure = ref<PlainDateTime | null>(createPlainDateTime(2026, 10, 24, 9, 30));
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Departure</span>
    <DateTimeField
      v-model="departure"
      name="departure"
      locale="en-GB"
      :hour-cycle="24"
      :min="createPlainDateTime(2026, 10, 1, 6, 0)"
      :aria-labelledby="labelId"
    />
    <output>{{ departure ? formatIsoDateTime(departure) : "No departure" }}</output>
  </div>
</template>
