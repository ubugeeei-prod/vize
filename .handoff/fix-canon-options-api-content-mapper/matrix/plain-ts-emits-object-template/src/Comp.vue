<script lang="ts">
export default {
  emits: { save: (n: number) => true },
}
</script>

<template>
  <div>
    <button @click="$emit('save', 1)">ok</button>
    <button @click="$emit('save', 'x')">bad</button>
    <button @click="$emit('other')">other</button>
  </div>
</template>
