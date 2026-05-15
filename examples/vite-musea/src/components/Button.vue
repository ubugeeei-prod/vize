<script setup lang="ts">
defineProps<{
  variant?: "default" | "primary" | "secondary";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
}>();
</script>

<template>
  <button
    class="btn"
    :class="[`btn--${variant ?? 'default'}`, `btn--${size ?? 'md'}`, { 'btn--disabled': disabled }]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.btn {
  border: 1px solid #c8c4b8;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  transition: all 0.15s ease;
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
  background: #e6e2d6;
  color: #121212;
}

.btn--default:hover:not(:disabled) {
  background: #ddd9cd;
  border-color: #9a9890;
}

.btn--primary {
  background: #121212;
  border-color: #121212;
  color: #e6e2d6;
}

.btn--primary:hover:not(:disabled) {
  background: #2a2a2a;
  border-color: #2a2a2a;
}

.btn--secondary {
  background: #6b5090;
  border-color: #6b5090;
  color: #e6e2d6;
}

.btn--secondary:hover:not(:disabled) {
  background: #5a4080;
  border-color: #5a4080;
}

.btn--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

<art title="Button" category="Components" status="ready">
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
