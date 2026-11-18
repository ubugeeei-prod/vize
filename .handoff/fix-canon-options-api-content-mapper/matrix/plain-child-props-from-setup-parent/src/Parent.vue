<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child title="ok" :size="2" @pick="() => {}" />
  <Child :title="1" />
  <Child />
</template>
