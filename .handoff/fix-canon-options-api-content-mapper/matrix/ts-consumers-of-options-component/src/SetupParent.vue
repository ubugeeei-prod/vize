<script setup lang="ts">
import { useTemplateRef, onMounted } from 'vue'
import Child from './Child.vue'
const child = useTemplateRef<InstanceType<typeof Child>>('child')
onMounted(() => {
  child.value?.toggle()
  child.value?.toggle('x')
  const n: number = child.value!.title
  return n
})
</script>

<template>
  <Child ref="child" title="t" />
</template>
