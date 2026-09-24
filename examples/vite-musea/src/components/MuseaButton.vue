<script setup lang="ts">
import "../theme.css";
defineProps<{
  variant?: "default" | "primary" | "secondary";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
}>();

defineArt("./MuseaButton.vue", {
  title: "Button",
  category: "Components",
  tags: ["button", "action", "form"],
  status: "ready",
});
</script>

<template>
  <button
    type="button"
    class="btn"
    :class="[`btn--${variant ?? 'default'}`, `btn--${size ?? 'md'}`, { 'btn--disabled': disabled }]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.btn {
  border: 1px solid var(--musea-line);
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease,
    color 0.15s ease;
}

.btn--sm {
  padding: 0.25rem 0.625rem;
  font-size: 0.75rem;
}

.btn--md {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
}

.btn--lg {
  padding: 0.625rem 1.5rem;
  font-size: 1rem;
}

.btn--default {
  background: var(--musea-paper);
  color: var(--musea-ink);
}

.btn--default:hover:not(:disabled) {
  background: var(--musea-surface);
  border-color: var(--musea-muted);
}

.btn--primary {
  background: var(--musea-ink);
  border-color: var(--musea-ink);
  color: var(--musea-paper);
}

.btn--primary:hover:not(:disabled) {
  background: var(--musea-ink-hover);
  border-color: var(--musea-ink-hover);
}

.btn--secondary {
  background: var(--musea-accent);
  border-color: var(--musea-accent);
  color: var(--musea-paper);
}

.btn--secondary:hover:not(:disabled) {
  background: var(--musea-accent-hover);
  border-color: var(--musea-accent-hover);
}

.btn--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

<art>
  <variant name="Default" default>
    <Self>Default Button</Self>
  </variant>
  <variant name="Primary">
    <Self variant="primary">Primary Button</Self>
  </variant>
  <variant name="Secondary">
    <Self variant="secondary">Secondary Button</Self>
  </variant>
  <variant name="Disabled">
    <Self disabled>Disabled Button</Self>
  </variant>
  <variant name="Sizes">
    <div style="display: flex; gap: 0.75rem; align-items: center">
      <Self size="sm" variant="primary">Small</Self>
      <Self size="md" variant="primary">Medium</Self>
      <Self size="lg" variant="primary">Large</Self>
    </div>
  </variant>
  <variant name="All Variants" skip-vrt>
    <div style="display: flex; gap: 1rem; flex-wrap: wrap">
      <Self>Default</Self>
      <Self variant="primary">Primary</Self>
      <Self variant="secondary">Secondary</Self>
      <Self disabled>Disabled</Self>
    </div>
  </variant>
</art>
