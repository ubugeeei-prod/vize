<!-- Controlled segmented time field with a fixed locale, business-hour bounds, and ISO output. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { createPlainTime, formatIsoTime, TimeField } from "../time-field.ts";
import type { PlainTime } from "../time-field.ts";

const start = ref<PlainTime | null>(createPlainTime(9, 30));
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Meeting start</span>
    <TimeField
      v-model="start"
      name="start"
      locale="en-US"
      :min="createPlainTime(8)"
      :max="createPlainTime(18)"
      :aria-labelledby="labelId"
    />
    <output>{{ start ? formatIsoTime(start) : "No time selected" }}</output>
  </div>
</template>
