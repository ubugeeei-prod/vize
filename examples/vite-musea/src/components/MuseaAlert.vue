<script setup lang="ts">
import "../theme.css";

defineProps<{
  variant?: "default" | "success" | "warning" | "error";
  closable?: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

function closeAlert() {
  emit("close");
}

defineArt("./MuseaAlert.vue", {
  title: "Alert",
  category: "Feedback",
  tags: ["alert", "message", "feedback"],
  status: "ready",
});
</script>

<template>
  <div class="alert" :class="`alert--${variant ?? 'default'}`" role="alert">
    <span class="alert-icon">
      <template v-if="variant === 'success'">&#10003;</template>
      <template v-else-if="variant === 'warning'">&#9888;</template>
      <template v-else-if="variant === 'error'">&#10005;</template>
      <template v-else>&#8505;</template>
    </span>
    <span class="alert-content">
      <slot />
    </span>
    <button
      v-if="closable"
      type="button"
      class="alert-close"
      aria-label="Dismiss alert"
      @click="closeAlert"
    >
      &times;
    </button>
  </div>
</template>

<style scoped>
.alert {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
  line-height: 1.5;
  border: 1px solid;
}

.alert--default {
  background: var(--musea-surface);
  border-color: var(--musea-line);
  color: var(--musea-text-subtle);
}

.alert--success {
  background: var(--musea-success-wash);
  border-color: var(--musea-success-line);
  color: var(--musea-success);
}

.alert--warning {
  background: var(--musea-warning-wash);
  border-color: var(--musea-warning-line);
  color: var(--musea-warning);
}

.alert--error {
  background: var(--musea-error-wash);
  border-color: var(--musea-error-line);
  color: var(--musea-error);
}

.alert-icon {
  flex-shrink: 0;
  font-size: 1rem;
  line-height: 1.3;
}

.alert-content {
  flex: 1;
}

.alert-close {
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.125rem;
  line-height: 1;
  padding: 0;
  color: inherit;
  opacity: 0.6;
  transition: opacity 0.15s ease;

  &:hover {
    opacity: 1;
  }
}
</style>

<art>
  <variant name="Default" default>
    <Self>This is a default informational alert.</Self>
  </variant>
  <variant name="Success">
    <Self variant="success">Operation completed successfully.</Self>
  </variant>
  <variant name="Warning">
    <Self variant="warning">Please review before proceeding.</Self>
  </variant>
  <variant name="Error">
    <Self variant="error">Something went wrong. Please try again.</Self>
  </variant>
  <variant name="With Close Button">
    <Self variant="success" closable>This alert can be dismissed.</Self>
  </variant>
</art>
