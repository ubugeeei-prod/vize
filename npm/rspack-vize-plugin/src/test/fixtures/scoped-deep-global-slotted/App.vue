<script setup lang="ts">
const msg: string = "deep, global, slotted";
</script>

<template>
  <div class="wrapper">
    <h1>{{ msg }}</h1>
    <slot />
  </div>
</template>

<style scoped>
.wrapper {
  padding: 1rem;
}

.wrapper :deep(.child) {
  color: red;
}

.wrapper :deep(.child .grandchild) {
  font-weight: bold;
}
/** :global test */
:global(.global-reset) {
  margin: 0;
  padding: 0;
}

.wrapper :global(.external-lib) {
  border: none;
}
/** :slotted test */
:slotted(div) {
  background: #f0f0f0;
}

:slotted(.slot-item) {
  padding: 0.5rem;
}

:slotted(.slot-item):hover {
  background: #e0e0e0;
}
</style>
