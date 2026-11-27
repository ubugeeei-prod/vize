<script setup lang="ts">
const greeting = ref("generated");
</script>

<template>
  <main>
    <h1 id="greeting">{{ greeting }}</h1>
    <NuxtLink to="/about">About</NuxtLink>
  </main>
</template>
