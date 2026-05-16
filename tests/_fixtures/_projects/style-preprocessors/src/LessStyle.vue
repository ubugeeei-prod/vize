<script setup lang="ts">
import { ref } from "vue";
const visible = ref(true);
</script>

<template>
  <div v-if="visible" class="box">
    <span class="label">Hello Less</span>
  </div>
</template>

<style lang="less">
@primary: #42b883;
@spacing: 8px;

.box {
  margin: @spacing;

  .label {
    color: @primary;
    padding: @spacing * 2;
  }
}
</style>
