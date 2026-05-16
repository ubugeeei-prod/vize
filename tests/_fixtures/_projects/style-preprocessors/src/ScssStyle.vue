<script setup lang="ts">
import { ref } from "vue";
const active = ref(true);
</script>

<template>
  <div class="container" :class="{ active }">
    <p class="text">Hello SCSS</p>
  </div>
</template>

<style lang="scss">
$primary: #42b883;
$padding: 16px;

.container {
  padding: $padding;

  &.active {
    border: 1px solid $primary;
  }

  .text {
    color: $primary;
    font-size: 14px;
  }
}
</style>
