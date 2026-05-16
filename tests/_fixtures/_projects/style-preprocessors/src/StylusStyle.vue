<script setup lang="ts">
import { ref } from "vue";
const message = ref("Hello Stylus");
</script>

<template>
  <div class="wrapper">
    <p class="content">{{ message }}</p>
  </div>
</template>

<style lang="stylus">
primary = #42b883
gap = 12px

.wrapper
  padding gap

  .content
    color primary
    font-weight bold
</style>
