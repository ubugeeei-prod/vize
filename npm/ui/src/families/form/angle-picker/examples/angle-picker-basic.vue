<!-- Controlled angle picker labelled by visible text, with the angle exposed to its slot. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { AnglePicker } from "../angle-picker.ts";

const direction = ref(90);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Gradient direction</span>
    <AnglePicker
      v-slot="{ angle }"
      v-model="direction"
      name="direction"
      :large-step="45"
      :aria-labelledby="labelId"
    >
      <span aria-hidden="true">{{ angle }}°</span>
    </AnglePicker>
    <output>{{ direction }} degrees</output>
  </div>
</template>
