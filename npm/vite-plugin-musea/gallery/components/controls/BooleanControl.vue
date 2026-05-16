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
  display: none;
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
  left: 2px;
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
  left: 16px;
  background: white;
}

.control-required {
  color: var(--musea-error);
}

.control-desc {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  margin-left: 2.5rem;
}
</style>
