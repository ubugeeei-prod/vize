<script setup lang="ts">
import Child from './Child.vue'
import { ref } from 'vue'
const base = ref({ color: 'red' })
const style = { ...base.value }
</script>

<template>
  <p :style="style"><Child /></p>
</template>
