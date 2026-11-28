<script setup lang="ts">
import { computed } from "vue";

import { formatStatistic } from "./statistic-format.ts";
import type { StatisticProps } from "./types.ts";

const {
  title,
  value,
  prefix = "",
  suffix = "",
  locale = "en-US",
  maximumFractionDigits = 0,
  notation = "standard",
} = defineProps<StatisticProps>();

const text = computed(() =>
  formatStatistic({
    value,
    locale,
    maximumFractionDigits,
    notation,
    prefix,
    suffix,
  }),
);
</script>

<template>
  <div role="group" data-vize-ui="statistic" :aria-label="title" :data-notation="notation">
    <p data-part="title">{{ title }}</p>
    <p data-part="value">{{ text }}</p>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
