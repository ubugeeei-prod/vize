<script setup lang="ts">
import { computed } from "vue";
import type { DesignToken } from "../../api";
import { resolveTokenPreview, type MuseaTokenPreviewConfig } from "../../../src/tokens/preview.js";
import SpacingPreview from "./SpacingPreview.vue";
import TypographyPreview from "./TypographyPreview.vue";

const props = withDefaults(
  defineProps<{
    tokenPath: string;
    token: DesignToken;
    tokenMap?: Record<string, DesignToken>;
  }>(),
  {
    tokenMap: () => ({}),
  },
);

const tokenPreviewConfig =
  typeof window === "undefined"
    ? undefined
    : (window as unknown as { __MUSEA_TOKEN_PREVIEWS__?: MuseaTokenPreviewConfig })
        .__MUSEA_TOKEN_PREVIEWS__;

const preview = computed(() =>
  resolveTokenPreview({
    tokenPath: props.tokenPath,
    token: props.token,
    tokenMap: props.tokenMap,
    config: tokenPreviewConfig,
  }),
);

const value = computed(() => preview.value.value);

const radiusStyle = computed(() => ({
  borderRadius: cssLength(value.value),
}));

const opacityStyle = computed(() => ({
  opacity: String(normalizeOpacity(value.value)),
}));

const zIndexLevel = computed(() => zLevel(value.value));

const zIndexStyle = computed(() => ({
  "--musea-z-preview-level": String(zIndexLevel.value),
  zIndex: String(2 + zIndexLevel.value),
}));

function cssLength(input: string | number): string {
  return typeof input === "number" ? `${input}px` : String(input);
}

function normalizeOpacity(input: string | number): number {
  const raw = String(input).trim();
  const parsed = raw.endsWith("%") ? Number.parseFloat(raw) / 100 : Number.parseFloat(raw);
  if (!Number.isFinite(parsed)) return 1;
  return Math.min(Math.max(parsed > 1 ? parsed / 100 : parsed, 0), 1);
}

function zLevel(input: string | number): number {
  const parsed = typeof input === "number" ? input : Number.parseFloat(input);
  if (!Number.isFinite(parsed)) return 2;
  if (parsed <= 0) return 0;
  if (parsed <= 10) return 1;
  if (parsed <= 100) return 2;
  if (parsed <= 1000) return 3;
  return 4;
}
</script>

<template>
  <div class="token-preview" :class="'token-preview--' + preview.kind">
    <div
      v-if="preview.kind === 'color'"
      class="color-swatch"
      :style="{ background: String(value) }"
    />
    <div v-else class="preview-compact">
      <SpacingPreview v-if="preview.kind === 'spacing'" :value="value" />
      <TypographyPreview
        v-else-if="preview.kind === 'fontSize'"
        :value="value"
        token-type="fontSize"
      />
      <TypographyPreview
        v-else-if="preview.kind === 'fontWeight'"
        :value="value"
        token-type="fontWeight"
      />
      <TypographyPreview
        v-else-if="preview.kind === 'lineHeight'"
        :value="value"
        token-type="lineHeight"
      />
      <div
        v-else-if="preview.kind === 'shadow'"
        class="shadow-swatch"
        :style="{ boxShadow: String(value) }"
      />
      <div v-else-if="preview.kind === 'radius'" class="radius-swatch" :style="radiusStyle" />
      <div v-else-if="preview.kind === 'opacity'" class="opacity-preview">
        <div class="opacity-fill" :style="opacityStyle" />
      </div>
      <div v-else-if="preview.kind === 'zIndex'" class="zindex-preview">
        <div class="zindex-layer zindex-layer--back" />
        <div class="zindex-layer zindex-layer--front" />
        <div class="zindex-layer zindex-layer--token" :style="zIndexStyle">
          {{ value }}
        </div>
      </div>
      <div v-else class="generic-preview">
        <span class="generic-value-icon">T</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.token-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 48px;
  padding: 0.75rem;
}

.token-preview--color {
  padding: 0;
}

.color-swatch {
  width: 100%;
  height: 64px;
}

.preview-compact {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 48px;
}

.shadow-swatch,
.radius-swatch,
.opacity-preview,
.generic-preview {
  width: 48px;
  height: 48px;
}

.shadow-swatch {
  border-radius: var(--musea-radius-md);
  background: var(--musea-bg);
}

.radius-swatch {
  border: 2px solid var(--musea-accent);
  background: transparent;
}

.opacity-preview {
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  overflow: hidden;
  background-color: var(--musea-bg);
  background-image:
    linear-gradient(45deg, var(--musea-border) 25%, transparent 25%),
    linear-gradient(-45deg, var(--musea-border) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, var(--musea-border) 75%),
    linear-gradient(-45deg, transparent 75%, var(--musea-border) 75%);
  background-position:
    0 0,
    0 6px,
    6px -6px,
    -6px 0;
  background-size: 12px 12px;
}

.opacity-fill {
  width: 100%;
  height: 100%;
  background: var(--musea-accent);
}

.zindex-preview {
  position: relative;
  width: 68px;
  height: 48px;
}

.zindex-layer {
  position: absolute;
  width: 34px;
  height: 26px;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm, 4px);
  background: var(--musea-bg);
}

.zindex-layer--back {
  left: 4px;
  top: 16px;
  z-index: 1;
}

.zindex-layer--front {
  left: 28px;
  top: 6px;
  z-index: 5;
  background: var(--musea-bg-tertiary);
}

.zindex-layer--token {
  left: calc(10px + var(--musea-z-preview-level) * 4px);
  top: calc(18px - var(--musea-z-preview-level) * 3px);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--musea-accent-contrast);
  background: var(--musea-accent);
  box-shadow: 0 calc(4px + var(--musea-z-preview-level) * 1px)
    calc(12px + var(--musea-z-preview-level) * 2px) rgba(0, 0, 0, 0.2);
  font-family: var(--musea-font-mono);
  font-size: 0.625rem;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.generic-preview {
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  color: var(--musea-text-muted);
}

.generic-value-icon {
  font-family: var(--musea-font-mono);
  font-size: 1rem;
  font-weight: 600;
}
</style>
