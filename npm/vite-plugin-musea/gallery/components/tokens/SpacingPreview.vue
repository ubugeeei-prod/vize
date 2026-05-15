<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  value: string | number;
}>();

const numericValue = computed(() => {
  if (typeof props.value === "number") return props.value;
  const parsed = parseFloat(props.value);
  return isNaN(parsed) ? 0 : parsed;
});

const label = computed(() => {
  if (typeof props.value === "number") return `${props.value}px`;
  return String(props.value);
});

const barWidth = computed(() => {
  const px = numericValue.value;
  // Cap at 200px for display
  return Math.min(Math.max(px, 2), 200);
});
</script>

<template>
  <div class="spacing-preview">
    <div class="spacing-bar" :style="{ width: barWidth + 'px' }" />
    <span class="spacing-label">{{ label }}</span>
  </div>
</template>

<style scoped>
.spacing-preview {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
}

.spacing-bar {
  height: 12px;
  min-width: 2px;
  background: var(--musea-accent);
  border-radius: 2px;
  transition: width 0.2s ease;
}

.spacing-label {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  font-family: var(--musea-font-mono);
  white-space: nowrap;
}
</style>
