<script setup lang="ts">
const message: string = "Hello from Rspack Vize";
</script>

<template>
  <div :class="$style.root">
    <span class="note">{{ message }}</span>
  </div>
</template>

<style module>
.root {
  color: royalblue;
}
</style>

<style scoped>
.note {
  color: tomato;
}
</style>
