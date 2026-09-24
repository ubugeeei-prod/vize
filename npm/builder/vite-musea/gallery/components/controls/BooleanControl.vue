<script setup lang="ts">
const model = defineModel<boolean>({ default: false });

defineProps<{
  label: string;
  description?: string;
  required?: boolean;
}>();
</script>

<template>
  <div class="control">
    <label class="control-label">
      <input v-model="model" type="checkbox" class="control-checkbox" />
      <span class="control-toggle" :class="{ active: model }" />
      {{ label }}
      <span v-if="required" class="control-required">*</span>
    </label>
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
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.control-checkbox {
  position: absolute;
  inline-size: 1px;
  block-size: 1px;
  padding: 0;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;

  &:focus-visible + .control-toggle {
    outline: 2px solid var(--musea-accent);
    outline-offset: 2px;
  }
}

.control-toggle {
  width: 32px;
  height: 18px;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: 9px;
  position: relative;
  transition: all var(--musea-transition);
  flex-shrink: 0;
}

.control-toggle::after {
  content: "";
  position: absolute;
  top: 2px;
  inset-inline-start: 2px;
  width: 12px;
  height: 12px;
  background: var(--musea-text-muted);
  border-radius: 50%;
  transition: all var(--musea-transition);
}

.control-toggle.active {
  background: var(--musea-accent);
  border-color: var(--musea-accent);
}

.control-toggle.active::after {
  inset-inline-start: 16px;
  background: white;
}

.control-required {
  color: var(--musea-error);
}

.control-desc {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  margin-inline-start: 2.5rem;
}
</style>
