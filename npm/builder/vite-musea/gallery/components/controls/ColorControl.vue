<script setup lang="ts">
const model = defineModel<string>({ default: "#000000" });

defineProps<{
  label: string;
  description?: string;
  required?: boolean;
}>();
</script>

<template>
  <div class="control">
    <label class="control-label">
      {{ label }}
      <span v-if="required" class="control-required">*</span>
    </label>
    <div class="control-color-row">
      <input v-model="model" type="color" class="control-color" />
      <input v-model="model" type="text" class="control-color-text" placeholder="#000000" />
    </div>
    <span v-if="description" class="control-desc">{{ description }}</span>
  </div>
</template>

<style scoped>
.control {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.control-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--musea-text-secondary);
}

.control-required {
  color: var(--musea-error);
}

.control-color-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.control-color {
  width: 32px;
  height: 32px;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 2px;
  cursor: pointer;
  background: transparent;
}

.control-color::-webkit-color-swatch-wrapper {
  padding: 0;
}

.control-color::-webkit-color-swatch {
  border: none;
  border-radius: 3px;
}

.control-color-text {
  flex: 1;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.375rem 0.625rem;
  color: var(--musea-text);
  font-size: 0.8125rem;
  font-family: var(--musea-font-mono);
  outline: none;
  transition: border-color var(--musea-transition);
}

.control-color-text:focus {
  border-color: var(--musea-accent);
}

.control-desc {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
}
</style>
