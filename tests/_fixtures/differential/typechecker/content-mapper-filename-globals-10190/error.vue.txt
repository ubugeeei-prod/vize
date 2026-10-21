<script setup lang="ts">
function fail(message: string): never {
  throw new Error(message);
}
</script>

<template>
  <button @click="fail('boom')">fail</button>
</template>
