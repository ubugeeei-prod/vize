<script setup lang="ts">
import { computed } from "vue";

import { useGalleryMetadata } from "../composables/useGalleryMetadata";

const props = defineProps<{ tag: string }>();
const { config } = useGalleryMetadata();
const presentation = computed(() => config.value.tags?.[props.tag]);
const style = computed(() => {
  const color = presentation.value?.color;
  return color ? { color, borderColor: color } : undefined;
});
</script>

<template>
  <span
    v-if="!presentation?.hidden"
    class="tag-badge"
    :class="`tag-${presentation?.variant ?? 'neutral'}`"
    :style="style"
  >
    #{{ presentation?.label ?? tag }}
  </span>
</template>

<style scoped>
.tag-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.375rem;
  border: 1px solid transparent;
  border-radius: 4px;
  font-size: 0.6875rem;
}

.tag-neutral {
  color: var(--musea-text-muted);
  background: var(--musea-bg-tertiary);
}

.tag-info {
  color: var(--musea-info);
  background: color-mix(in srgb, var(--musea-info) 14%, transparent);
}

.tag-success {
  color: var(--musea-success);
  background: color-mix(in srgb, var(--musea-success) 14%, transparent);
}

.tag-warning {
  color: var(--musea-warning);
  background: color-mix(in srgb, var(--musea-warning) 14%, transparent);
}

.tag-error {
  color: var(--musea-error);
  background: color-mix(in srgb, var(--musea-error) 14%, transparent);
}
</style>
