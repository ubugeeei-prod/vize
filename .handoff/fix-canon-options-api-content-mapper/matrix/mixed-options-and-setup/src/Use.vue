<script lang="ts">
import { defineComponent } from 'vue'
import Comp from './Comp.vue'
export default defineComponent({
  components: { Comp },
  methods: { onPick(n: number) { return n } },
})
</script>

<template>
  <Comp label="x" @pick="onPick" />
  <Comp :label="1" />
</template>
