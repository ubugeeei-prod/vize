<script setup lang="ts">
const model = defineModel<unknown>();

defineProps<{
  label: string;
  description?: string;
  required?: boolean;
  options: Array<{ label: string; value: unknown }>;
}>();
</script>

<template>
  <div class="control">
    <label class="control-label">
      {{ label }}
      <span v-if="required" class="control-required">*</span>
    </label>
    <select
      class="control-select"
      :value="JSON.stringify(model)"
      @change="model = JSON.parse(($event.target as HTMLSelectElement).value)"
    >
      <option v-for="opt in options" :key="String(opt.value)" :value="JSON.stringify(opt.value)">
        {{ opt.label }}
      </option>
    </select>
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

.control-select {
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.375rem 0.625rem;
  color: var(--musea-text);
  font-size: 0.8125rem;
  outline: none;
  transition: border-color var(--musea-transition);
  cursor: pointer;
}

.control-select:focus {
  border-color: var(--musea-accent);
}

.control-desc {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
}
</style>
