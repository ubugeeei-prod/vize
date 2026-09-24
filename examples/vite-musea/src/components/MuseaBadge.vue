<script setup lang="ts">
import "../theme.css";
defineProps<{
  variant?: "default" | "success" | "warning" | "error" | "info";
  size?: "sm" | "md" | "lg";
}>();

defineArt("./MuseaBadge.vue", {
  title: "Badge",
  category: "Components",
  tags: ["badge", "tag", "label"],
  status: "ready",
});
</script>

<template>
  <span class="badge" :class="[`badge--${variant ?? 'default'}`, `badge--${size ?? 'md'}`]">
    <slot />
  </span>
</template>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  font-weight: 600;
  border-radius: 9999px;
  white-space: nowrap;
}

.badge--sm {
  padding: 0.125rem 0.5rem;
  font-size: 0.625rem;
}

.badge--md {
  padding: 0.25rem 0.625rem;
  font-size: 0.75rem;
}

.badge--lg {
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
}

.badge--default {
  background: var(--musea-neutral-200);
  color: var(--musea-text-subtle);
}

.badge--success {
  background: var(--musea-success-badge);
  color: var(--musea-success);
}

.badge--warning {
  background: var(--musea-warning-badge);
  color: var(--musea-warning);
}

.badge--error {
  background: var(--musea-error-badge);
  color: var(--musea-error);
}

.badge--info {
  background: var(--musea-info-badge);
  color: var(--musea-info);
}
</style>

<art>
  <variant name="Default" default>
    <Self>Default</Self>
  </variant>
  <variant name="Variants">
    <div style="display: flex; gap: 0.5rem; flex-wrap: wrap; align-items: center">
      <Self>Default</Self>
      <Self variant="success">Success</Self>
      <Self variant="warning">Warning</Self>
      <Self variant="error">Error</Self>
      <Self variant="info">Info</Self>
    </div>
  </variant>
  <variant name="Sizes">
    <div style="display: flex; gap: 0.5rem; align-items: center">
      <Self size="sm" variant="info">Small</Self>
      <Self size="md" variant="info">Medium</Self>
      <Self size="lg" variant="info">Large</Self>
    </div>
  </variant>
</art>
