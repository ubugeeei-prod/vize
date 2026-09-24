<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { mdiOpenInNew, mdiClose } from "@mdi/js";
import { useAddons } from "../composables/useAddons";
import { getPreviewUrl } from "../api";
import { safeUrl } from "../utils/safeUrl";
import MdiIcon from "./MdiIcon.vue";

const { fullscreenVariant, closeFullscreen } = useAddons();

const previewUrl = computed(() => {
  if (!fullscreenVariant.value) return "";
  return getPreviewUrl(fullscreenVariant.value.artPath, fullscreenVariant.value.variantName);
});

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") closeFullscreen();
}

function openInNewTab() {
  const url = safeUrl(previewUrl.value);
  if (url) window.open(url, "_blank", "noopener,noreferrer");
}

onMounted(() => document.addEventListener("keydown", onKeydown));
onUnmounted(() => document.removeEventListener("keydown", onKeydown));
</script>

<template>
  <Teleport to="body">
    <div v-if="fullscreenVariant" class="fullscreen-overlay">
      <button
        type="button"
        class="fullscreen-backdrop"
        aria-label="Close fullscreen preview"
        @click="closeFullscreen"
      />
      <div class="fullscreen-container">
        <div class="fullscreen-header">
          <span class="fullscreen-title">{{ fullscreenVariant.variantName }}</span>
          <div class="fullscreen-actions">
            <button
              type="button"
              class="fullscreen-action-btn"
              title="Open in new tab"
              @click="openInNewTab"
            >
              <MdiIcon :path="mdiOpenInNew" :size="16" />
            </button>
            <button
              type="button"
              class="fullscreen-close-btn"
              title="Close (Esc)"
              @click="closeFullscreen"
            >
              <MdiIcon :path="mdiClose" :size="18" />
            </button>
          </div>
        </div>
        <iframe
          class="fullscreen-iframe"
          :src="safeUrl(previewUrl)"
          :title="fullscreenVariant.variantName"
          sandbox="allow-scripts allow-same-origin allow-forms"
        />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.fullscreen-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--musea-z-fullscreen, 9999);
  background: var(--musea-overlay);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  animation: fadeIn 0.15s ease;
}

.fullscreen-backdrop {
  position: absolute;
  inset: 0;
  border: 0;
  background: transparent;
  cursor: default;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .fullscreen-overlay {
    animation: none;
  }
}

.fullscreen-container {
  position: relative;
  width: 100%;
  height: 100%;
  max-width: 1600px;
  display: flex;
  flex-direction: column;
  border-radius: var(--musea-radius-lg);
  overflow: hidden;
  box-shadow: var(--musea-shadow);
}

.fullscreen-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--musea-bg-secondary);
  border-bottom: 1px solid var(--musea-border);
  flex-shrink: 0;
}

.fullscreen-title {
  font-weight: 600;
  font-size: 0.875rem;
  color: var(--musea-text);
}

.fullscreen-actions {
  display: flex;
  gap: 0.5rem;
}

.fullscreen-action-btn,
.fullscreen-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: var(--musea-bg-tertiary);
  color: var(--musea-text-muted);
  cursor: pointer;
  transition: all var(--musea-transition);
}

.fullscreen-action-btn:hover,
.fullscreen-close-btn:hover {
  background: var(--musea-bg-elevated);
  color: var(--musea-text);
}

.fullscreen-iframe {
  flex: 1;
  width: 100%;
  border: none;
  background: var(--musea-preview-canvas, #fff);
}
</style>
