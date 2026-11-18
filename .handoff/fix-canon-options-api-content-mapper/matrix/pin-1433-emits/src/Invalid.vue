<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  emits: {
    save: (id: number) => id > 0,
    close: null,
  },
})
</script>

<template>
  <button @click="$emit('save', 'x')">save</button>
  <button @click="$emit('missing')">missing</button>
</template>
