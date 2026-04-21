<script setup lang="ts">
import type { ArtVariant } from "../../src/types/index.js";

defineProps<{
  variants: ArtVariant[];
  selectedVariant: string;
  sectionIds: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: "select", variantName: string): void;
}>();
</script>

<template>
  <nav class="variant-toc" aria-label="Variant table of contents">
    <div class="variant-toc-header">
      <p class="variant-toc-eyebrow">Variants</p>
      <p class="variant-toc-count">{{ variants.length }} sections</p>
    </div>

    <div class="variant-toc-list">
      <button
        v-for="(variant, index) in variants"
        :key="variant.name"
        type="button"
        class="variant-toc-item"
        :class="{ 'variant-toc-item--active': variant.name === selectedVariant }"
        :aria-controls="sectionIds[variant.name]"
        :aria-current="variant.name === selectedVariant ? 'true' : undefined"
        @click="emit('select', variant.name)"
      >
        <span class="variant-toc-index">{{ String(index + 1).padStart(2, "0") }}</span>

        <span class="variant-toc-body">
          <span class="variant-toc-name">{{ variant.name }}</span>
          <span class="variant-toc-caption">
            {{ variant.isDefault ? "Default variant" : `Section ${index + 1}` }}
          </span>
        </span>

        <span v-if="variant.isDefault" class="variant-toc-badge">Default</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.variant-toc {
  position: sticky;
  top: calc(var(--musea-header-height) + 1rem);
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  padding: 1rem;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-lg);
}

.variant-toc-header {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.variant-toc-eyebrow {
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--musea-text-muted);
}

.variant-toc-count {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--musea-text);
}

.variant-toc-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  max-height: calc(100vh - var(--musea-header-height) - 4rem);
  overflow-y: auto;
}

.variant-toc-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  padding: 0.75rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--musea-radius-md);
  color: var(--musea-text-muted);
  cursor: pointer;
  text-align: left;
  transition: all var(--musea-transition);
}

.variant-toc-item:hover {
  background: var(--musea-bg-tertiary);
  border-color: var(--musea-border);
  color: var(--musea-text);
}

.variant-toc-item--active {
  background: var(--musea-accent-subtle);
  border-color: rgba(163, 72, 40, 0.4);
  color: var(--musea-text);
  box-shadow: inset 3px 0 0 var(--musea-accent);
}

.variant-toc-index {
  flex-shrink: 0;
  min-width: 2.125rem;
  font-size: 0.75rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: inherit;
}

.variant-toc-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.variant-toc-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.875rem;
  font-weight: 600;
}

.variant-toc-caption {
  font-size: 0.75rem;
  color: var(--musea-text-muted);
}

.variant-toc-item--active .variant-toc-caption {
  color: var(--musea-text-secondary);
}

.variant-toc-badge {
  flex-shrink: 0;
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 0.25rem 0.5rem;
  border-radius: 999px;
  background: var(--musea-accent);
  color: white;
}

@media (max-width: 960px) {
  .variant-toc {
    position: static;
  }

  .variant-toc-list {
    max-height: none;
  }
}
</style>
