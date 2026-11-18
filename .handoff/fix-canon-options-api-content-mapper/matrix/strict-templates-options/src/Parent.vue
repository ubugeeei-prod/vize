<script lang="ts">
import { defineComponent } from 'vue'
import Child from './Child.vue'
export default defineComponent({ components: { Child } })
</script>

<template>
  <Child title="a" @pick="() => {}" />
  <Child title="a" unknown-prop="x" @unknown-event="() => {}" />
  <NotRegistered />
</template>
