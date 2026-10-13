<script setup>
import { ref } from 'vue'

const page = ref(1)
const pages = ref(3)
const update = (value) => value
</script>

<template lang="pug">
nav
  button(@click=" e => update(e)" :disabled="page === 1 && pages > 1") prev
  span(v-if="page < pages && pages > 1") {{ page }} / {{ pages }}
  input(@input="() => update(page)" :title="page > 1 ? '<' : '>'")
</template>
