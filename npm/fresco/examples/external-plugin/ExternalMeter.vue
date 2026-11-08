<script setup lang="ts">
import { computed } from "vue";

import { externalMeterTheme } from "./provider.js";

const { value, width = 20 } = defineProps<{ value: number; width?: number }>();
const theme = externalMeterTheme.use();
const normalized = computed(() => Math.max(0, Math.min(1, value)));
const filledWidth = computed(() => Math.round(normalized.value * width));
const meter = computed(
  () => theme.filled.repeat(filledWidth.value) + theme.empty.repeat(width - filledWidth.value),
);
</script>

<template>
  <text :text="meter" :fg="theme.foreground" />
</template>
