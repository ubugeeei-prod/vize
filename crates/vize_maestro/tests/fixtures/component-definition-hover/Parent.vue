<script setup lang="ts">
import Child from './Child.vue'
const attrs = {}
</script>
<template>
  <Child v-bind="attrs" message="usage-only" @save="() => {}">
    <template #provided>usage slot</template>
  </Child>
</template>
