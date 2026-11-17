<script setup lang="ts" generic="T extends string">
import Pick from "./Pick.vue";

defineProps<{ value: T }>();
</script>

<template>
  <Pick :value="value">
    <template #label="{ value: picked }">{{ picked }}</template>
    <template #caption>no such slot</template>
  </Pick>
</template>
