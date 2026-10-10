<script setup lang="ts">
import { computed, useId } from "vue";

defineOptions({ inheritAttrs: false });
const props = defineProps<{ label: string; modelValue?: unknown }>();
const inputId = useId();
const retained = computed(() =>
  typeof props.modelValue === "string" ? props.modelValue : JSON.stringify(props.modelValue),
);
</script>

<template>
  <div class="unsupported-prop-control">
    <label :for="inputId">{{ label }}</label>
    <input :id="inputId" :value="retained" type="text" disabled />
    <span>Retained only: Vue cannot apply this prop or include it in copied usage.</span>
  </div>
</template>

<style scoped>
.unsupported-prop-control {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  color: var(--musea-text-secondary);
  font-size: 0.75rem;
}
.unsupported-prop-control input {
  padding: 0.375rem 0.625rem;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-text-muted);
}
.unsupported-prop-control span {
  font-size: 0.6875rem;
}
</style>
