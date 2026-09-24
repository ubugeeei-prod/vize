<!-- Controlled range slider labelled by a native label element. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Slider } from "../slider.ts";

const volume = ref(40);
const sliderId = useId();
</script>

<template>
  <div>
    <label :for="sliderId">Volume</label>
    <Slider :id="sliderId" v-model="volume" name="volume" :step="5" />
    <output :for="sliderId">{{ volume }}%</output>
  </div>
</template>
