<!-- Volume knob bound with v-model, labelled by visible text and announcing a percentage. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Knob } from "../knob.ts";

const volume = ref(40);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Master volume</span>
    <Knob
      v-slot="{ value }"
      v-model="volume"
      name="volume"
      :min="0"
      :max="100"
      :aria-labelledby="labelId"
      :get-value-text="(value: number) => `${value}%`"
    >
      <span aria-hidden="true">{{ value }}%</span>
    </Knob>
  </div>
</template>
