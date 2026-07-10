<script setup lang="ts">
const msg: string = "multiple style blocks";
</script>

<template>
  <div class="multi">
    <p :class="$style.modular">{{ msg }}</p>
    <span class="unscoped-text">unscoped</span>
  </div>
</template>

<!-- Block 0: scoped -->
<style scoped>
.multi {
  padding: 1rem;
}

.multi :deep(.nested) {
  color: green;
}
</style>

<!-- Block 1: CSS Modules -->
<style module>
.modular {
  font-weight: bold;
  color: purple;
}
</style>

<!-- Block 2: unscoped global -->
<style>
.unscoped-text {
  font-style: italic;
}
</style>
