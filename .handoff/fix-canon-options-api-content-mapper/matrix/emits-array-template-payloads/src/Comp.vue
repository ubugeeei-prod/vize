<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  emits: ['open', 'close'],
  methods: { fire() { this.$emit('open', 1, 2); this.$emit('nope') } },
})
</script>

<template>
  <div>
    <button @click="$emit('open', 1)">o</button>
    <button @click="$emit('close')">c</button>
    <button @click="$emit('other')">x</button>
  </div>
</template>
