<script setup lang="ts">
import Comp from './Comp.vue'
</script>

<template>
  <Comp a="1" :b="2" />
  <Comp :c="3" />
</template>
