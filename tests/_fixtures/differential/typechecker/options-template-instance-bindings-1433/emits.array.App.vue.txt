<script lang="ts">
export default {
  emits: ['open', 'close'],
}
</script>

<template>
  <button @click="$emit('open', 1)">open</button>
  <button @click="$emit('other')">other</button>
</template>
