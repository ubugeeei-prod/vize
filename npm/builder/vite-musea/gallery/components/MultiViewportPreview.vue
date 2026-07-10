<script setup lang="ts">
import { MULTI_VIEWPORT_PRESETS } from "../composables/useAddons";
import { getPreviewUrl } from "../api";

defineProps<{
  artPath: string;
  variantName: string;
}>();
</script>

<template>
  <div class="multi-viewport">
    <div v-for="preset in MULTI_VIEWPORT_PRESETS" :key="preset.name" class="multi-viewport-item">
      <div class="multi-viewport-label">
        <span class="multi-viewport-name">{{ preset.name }}</span>
        <span class="multi-viewport-size">{{ preset.width }} x {{ preset.height }}</span>
      </div>
      <div class="multi-viewport-frame" :style="{ width: preset.width }">
        <iframe
          :src="getPreviewUrl(artPath, variantName)"
          :title="`${variantName} - ${preset.name}`"
          :style="{ width: preset.width, height: preset.height }"
          loading="lazy"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.multi-viewport {
  display: flex;
  gap: 1.5rem;
  overflow-x: auto;
  padding: 1rem 0;
}

.multi-viewport-item {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.multi-viewport-label {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
}

.multi-viewport-name {
  font-weight: 600;
  font-size: 0.75rem;
  color: var(--musea-text);
}

.multi-viewport-size {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  font-family: var(--musea-font-mono, monospace);
}

.multi-viewport-frame {
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  overflow: hidden;
  max-height: 500px;
}

.multi-viewport-frame iframe {
  border: none;
  background: #fff;
  display: block;
}
</style>
