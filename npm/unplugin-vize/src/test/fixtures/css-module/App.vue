<script setup lang="ts">
const message: string = "CSS module fixture";
</script>

<template>
  <div :class="$style.root">{{ message }}</div>
</template>

<style module>
.root {
  color: seagreen;
}
</style>
