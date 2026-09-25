<!-- Storage usage meter labelled by visible text, with low and high thresholds and fallback text. -->
<script setup lang="ts">
import { useId } from "vue";

import { Meter } from "../meter.ts";

const labelId = useId();
const usedGb = 42;
</script>

<template>
  <div>
    <span :id="labelId">Storage used</span>
    <Meter
      v-slot="{ percent }"
      :aria-labelledby="labelId"
      :value="usedGb"
      :min="0"
      :max="50"
      :low="25"
      :high="40"
      :optimum="10"
    >
      {{ percent }}% used
    </Meter>
    <span>{{ usedGb }} GB of 50 GB</span>
  </div>
</template>
