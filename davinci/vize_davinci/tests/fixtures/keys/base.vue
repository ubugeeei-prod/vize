<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
const label = 'counter'
</script>

<template>
  <div class="counter" :title="label">
    <button @click="count++">{{ count }}</button>
    <p v-if="count > 3">many</p>
    <p v-else>few</p>
  </div>
</template>

<style scoped>
.counter { color: red; }
</style>

<style scoped>
.counter { color: red; }
</style>

<i18n lang="json">
{ "en": { "hello": "Hello" } }
</i18n>
