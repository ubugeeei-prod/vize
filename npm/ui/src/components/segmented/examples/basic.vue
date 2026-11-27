<!-- Billing period as an exclusive segmented radiogroup. -->
<script setup lang="ts">
import { ref } from "vue";

import { Segmented, SegmentedItem } from "../index.ts";

const period = ref("day");
</script>

<template>
  <div>
    <Segmented v-model="period" label="Billing">
      <SegmentedItem value="day">Day</SegmentedItem>
      <SegmentedItem value="week">Week</SegmentedItem>
      <SegmentedItem value="month">Month</SegmentedItem>
    </Segmented>
    <output>{{ period }}</output>
  </div>
</template>
