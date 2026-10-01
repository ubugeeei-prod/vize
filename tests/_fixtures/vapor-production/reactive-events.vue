<script setup>
import { ref, computed } from 'vue';
import { record } from './fixture';
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
<template>
  <main>
    <button @click="count++; record(count)">{{ count }}:{{ doubled }}</button>
  </main>
</template>
