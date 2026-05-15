<script setup lang="ts">
import { ref } from "vue";
const label = ref("Hello CSS Modules");
</script>

<template>
  <div :class="$style.root">
    <span :class="$style.text">{{ label }}</span>
  </div>
</template>

<style lang="scss" module>
$accent: #35495e;

.root {
  display: flex;
  align-items: center;
}

.text {
  color: $accent;
  font-size: 16px;
}
</style>
