<script setup lang="ts">
const accent = "red";
</script>

<template>
  <div class="root">v-bind</div>
</template>

<style scoped lang="scss">
.root {
  color: v-bind(accent);
}
</style>

<style>
.global-accent {
  border-color: v-bind(accent);
}
</style>
