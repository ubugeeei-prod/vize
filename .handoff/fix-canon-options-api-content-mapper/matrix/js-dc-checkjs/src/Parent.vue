<script>
import { defineComponent } from 'vue'
import Comp from './Comp.vue'
export default defineComponent({ components: { Comp } })
</script>

<template>
  <Comp msg="a" />
  <Comp :msg="1" />
  <Comp />
</template>
