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
  <button @click="$emit('save', 1)">save</button>
  <button @click="$emit('close')">close</button>
</template>
