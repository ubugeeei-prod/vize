<script setup lang="ts">
defineProps<{ label: string; required?: boolean }>();
defineOptions({ inheritAttrs: false });
</script>

<template>
  <div class="function-control">
    <span class="function-label">{{ label }}<span v-if="required"> *</span></span>
    <p role="note">Function props retain their component value and cannot be edited as JSON.</p>
  </div>
</template>

<style scoped>
.function-control {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  color: var(--musea-text-secondary);
  font-size: 0.75rem;
}
.function-label {
  font-weight: 600;
}
p {
  margin: 0;
  font-size: 0.6875rem;
}
</style>
