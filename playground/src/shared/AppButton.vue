<script setup lang="ts">
defineProps<{
  variant?: "default" | "primary" | "secondary";
  disabled?: boolean;
}>();
</script>

<template>
  <button
    type="button"
    class="btn"
    :class="[`btn--${variant ?? 'default'}`, { 'btn--disabled': disabled }]"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.btn {
  --button-border: #ccc;

  padding: 0.5rem 1rem;
  border: 1px solid var(--button-border);
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  transition: all 0.2s;
  background: var(--button-background);
  color: var(--button-foreground);
}

.btn--default {
  --button-background: #fff;
  --button-foreground: #333;
}

.btn--default:hover:not(:disabled) {
  --button-background: #f5f5f5;
}

.btn--primary {
  --button-background: #42b883;
  --button-border: #42b883;
  --button-foreground: #fff;
}

.btn--primary:hover:not(:disabled) {
  --button-background: #33a06f;
}

.btn--secondary {
  --button-background: #35495e;
  --button-border: #35495e;
  --button-foreground: #fff;
}

.btn--secondary:hover:not(:disabled) {
  --button-background: #2c3e50;
}

.btn--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
