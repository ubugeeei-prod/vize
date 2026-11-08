<script setup lang="ts">
import { computed } from "vue";

import { externalMeterTheme } from "./provider.js";

const { value, width = 20 } = defineProps<{ value: number; width?: number }>();
const theme = externalMeterTheme.use();
const safeWidth = computed(() => (Number.isFinite(width) ? Math.max(0, Math.floor(width)) : 0));
const normalized = computed(() => (Number.isFinite(value) ? Math.max(0, Math.min(1, value)) : 0));
const filledWidth = computed(() => Math.round(normalized.value * safeWidth.value));
const meter = computed(
  () =>
    theme.filled.repeat(filledWidth.value) +
    theme.empty.repeat(safeWidth.value - filledWidth.value),
);
</script>

<template>
  <text :text="meter" :fg="theme.foreground" />
</template>
