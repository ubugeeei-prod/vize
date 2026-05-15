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
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0 0 0 1.125rem;
  background: transparent;
  border-left: 1px solid var(--musea-border-subtle);
}

.variant-toc-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0 0.5rem 0 0.25rem;
}

.variant-toc-eyebrow {
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--musea-text-muted);
}

.variant-toc-count {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--musea-text-muted);
  white-space: nowrap;
}

.variant-toc-list {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  max-height: calc(100vh - var(--musea-header-height) - 4rem);
  overflow-y: auto;
  padding-right: 0.125rem;
}

.variant-toc-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  padding: 0.6875rem 0.75rem 0.6875rem 0.5rem;
  background: transparent;
  border: none;
  border-radius: var(--musea-radius-md);
  color: var(--musea-text-muted);
  cursor: pointer;
  text-align: left;
  transition:
    background var(--musea-transition),
    color var(--musea-transition);
}

.variant-toc-item:hover {
  background: var(--musea-bg-tertiary);
  color: var(--musea-text);
}

.variant-toc-item--active {
  background: var(--musea-accent-subtle);
  color: var(--musea-text);
}

.variant-toc-index {
  flex-shrink: 0;
  min-width: 1.875rem;
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
  padding: 0.1875rem 0.4375rem;
  border-radius: 999px;
  background: var(--musea-accent-subtle);
  color: var(--musea-accent);
}

@media (max-width: 960px) {
  .variant-toc {
    position: static;
    padding-left: 0;
    border-left: none;
    border-top: 1px solid var(--musea-border-subtle);
    padding-top: 0.875rem;
  }

  .variant-toc-list {
    max-height: none;
  }
}
</style>
