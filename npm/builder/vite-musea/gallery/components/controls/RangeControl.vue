<script setup lang="ts">
import { useId } from "vue";

const model = defineModel<number>({ default: 0 });

defineProps<{
  label: string;
  description?: string;
  required?: boolean;
  min?: number;
  max?: number;
  step?: number;
}>();

const inputId = useId();
</script>

<template>
  <div class="control">
    <label class="control-label" :for="inputId">
      {{ label }}
      <span v-if="required" class="control-required">*</span>
      <span class="control-value">{{ model }}</span>
    </label>
    <input
      :id="inputId"
      v-model.number="model"
      type="range"
      class="control-range"
      :min="min ?? 0"
      :max="max ?? 100"
      :step="step ?? 1"
      :required="required"
    />
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
}

.control-required {
  color: var(--musea-error);
}

.control-value {
  margin-inline-start: auto;
  font-family: var(--musea-font-mono);
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  background: var(--musea-bg-tertiary);
  padding: 0.0625rem 0.375rem;
  border-radius: 4px;
}

.control-range {
  width: 100%;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--musea-bg-tertiary);
  border-radius: 2px;
  outline: none;
}

.control-range::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--musea-accent);
  cursor: pointer;
  border: 2px solid var(--musea-bg-primary);
}

.control-desc {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
}
</style>
