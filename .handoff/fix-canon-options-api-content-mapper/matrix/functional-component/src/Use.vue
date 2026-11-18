<script lang="ts">
import { defineComponent } from 'vue'
import { Fn } from './Fn'
export default defineComponent({
  components: { Fn },
  methods: { onPick(n: number) { return n } },
})
</script>

<template>
  <Fn msg="a" @pick="onPick" />
  <Fn :msg="1" />
</template>
