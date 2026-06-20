<script setup lang="ts" vapor>
const title: string = "Vapor with scoped style";
</script>

<template>
  <div class="scoped-app">
    <h1>{{ title }}</h1>
  </div>
</template>

<style scoped>
.scoped-app {
  color: tomato;
}

h1 {
  font-size: 2rem;
}
</style>
