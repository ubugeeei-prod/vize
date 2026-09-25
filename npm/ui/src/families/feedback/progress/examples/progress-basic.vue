<!-- Native determinate progressbar labelled by visible text. -->
<script setup lang="ts">
import { useId } from "vue";

import { Progress } from "../progress.ts";

const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Uploading photos</span>
    <Progress :value="40" :max="100" :aria-labelledby="labelId">
      <template #default="{ percent }">{{ percent }}%</template>
    </Progress>
  </div>
</template>
