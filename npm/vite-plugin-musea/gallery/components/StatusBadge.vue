<script setup lang="ts">
defineProps<{
  status: "draft" | "ready" | "deprecated";
}>();
</script>

<template>
  <span class="status-badge" :class="'status-' + status">
    {{ status }}
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  font-size: 0.625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 0.1875rem 0.5rem;
  border-radius: 4px;
}

.status-draft {
  background: rgba(251, 191, 36, 0.15);
  color: var(--musea-warning);
}

.status-ready {
  background: rgba(74, 222, 128, 0.15);
  color: var(--musea-success);
}

.status-deprecated {
  background: rgba(248, 113, 113, 0.15);
  color: var(--musea-error);
}
</style>
