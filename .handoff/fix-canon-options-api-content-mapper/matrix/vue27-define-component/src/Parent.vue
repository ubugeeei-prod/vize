<script lang="ts">
import { defineComponent } from 'vue'
import Comp from './Comp.vue'
export default defineComponent({ components: { Comp } })
</script>

<template>
  <div>
    <Comp title="a" />
    <Comp :title="1" />
  </div>
</template>
