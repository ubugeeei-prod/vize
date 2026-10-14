<script setup lang="ts">
const title = "Scoped preprocessor";
</script>

<template>
  <div class="root">
    <p class="title">{{ title }}</p>
  </div>
</template>

<style scoped lang="scss">
$accent: red;

.root {
  color: $accent;

  .title {
    color: blue;
  }

  :deep(.inner) {
    color: green;
  }
}
</style>

<style scoped>
.plain {
  color: black;
}
</style>
