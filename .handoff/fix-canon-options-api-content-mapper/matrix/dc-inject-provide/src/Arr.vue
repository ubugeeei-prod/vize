<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  inject: ['foo', 'bar'],
})
</script>

<template>
  <div>{{ foo }} {{ bar }} {{ baz }}</div>
</template>
