<script setup lang="ts">
import { ref } from 'vue'
import Child from './Child.vue'
const n = ref(1)
function onPick(v: number) { n.value = v }
</script>

<template>
  <Child title="ok" :size="n" @pick="onPick" />
  <Child :title="n" />
  <Child />
  <Child title="x" kind="c" />
  <Child title="x" @pick="(v: string) => v" />
</template>
