<script setup lang="ts">
import { useId } from "vue";

const model = defineModel<string>({ default: "" });

defineProps<{
  label: string;
  description?: string;
  required?: boolean;
}>();

const inputId = useId();
</script>

<template>
  <div class="control">
    <label class="control-label" :for="inputId">
      {{ label }}
      <span v-if="required" class="control-required">*</span>
    </label>
    <input :id="inputId" v-model="model" type="text" class="control-input" :required="required" />
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

.control-input {
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.375rem 0.625rem;
  color: var(--musea-text);
  font-size: 0.8125rem;
  outline: none;
  transition: border-color var(--musea-transition);
}

.control-input:focus {
  border-color: var(--musea-accent);
}

.control-desc {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
}
</style>
