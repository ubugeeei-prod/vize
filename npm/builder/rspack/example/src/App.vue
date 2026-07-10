<script setup lang="ts">
import { ref } from "vue";
import ExampleCounter from "./ExampleCounter.vue";

const message = ref("Hello from Vize plugin with Rspack!");
</script>

<template>
  <div class="app">
    <h1>
      {{ message }}
    </h1>
    <ExampleCounter />
  </div>
</template>

<style scoped>
.app {
  font-family: system-ui, sans-serif;
  text-align: center;
  padding: 2rem;
}

h1 {
  color: #42b883;
}
</style>
