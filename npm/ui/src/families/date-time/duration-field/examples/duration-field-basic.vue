<!-- Duration field editing hours and minutes and submitting an ISO 8601 duration. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { DurationField, formatIsoDuration } from "../duration-field.ts";
import type { DurationValue } from "../duration-field.ts";

const estimate = ref<DurationValue | null>({ hours: 1, minutes: 30 });
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Estimated effort</span>
    <DurationField
      v-model="estimate"
      name="estimate"
      locale="en-US"
      :fields="['hours', 'minutes']"
      unit-display="short"
      :aria-labelledby="labelId"
    />
    <output>{{ estimate ? formatIsoDuration(estimate) : "Not set" }}</output>
  </div>
</template>
