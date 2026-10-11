<script setup lang="ts" vapor>
import { computed } from "vue";

const props = defineProps<{ tier: number }>();
const color = computed(() => (props.tier === 1 ? "rgb(200, 0, 0)" : "rgb(0, 0, 200)"));
</script>

<template>
  <svg id="icon" class="icon" :class="`tier-${tier}`" :style="{ color }" viewBox="0 0 64 64" width="64" height="64">
    <g v-if="tier === 1" id="mark-1"><circle cx="32" cy="32" r="16" fill="currentColor" /></g>
    <g v-else id="mark-other"><rect x="16" y="16" width="32" height="32" fill="currentColor" /></g>
  </svg>
</template>
