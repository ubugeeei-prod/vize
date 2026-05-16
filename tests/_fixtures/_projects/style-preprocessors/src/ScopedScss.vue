<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>

<template>
  <div class="scoped-container">
    <span>{{ count }}</span>
  </div>
</template>

<style lang="scss" scoped>
$bg: #f0f0f0;

.scoped-container {
  background: $bg;

  span {
    font-weight: bold;
  }
}
</style>
