<script setup lang="ts">
import { ref } from "vue";
import ExampleCounter from "./ExampleCounter.vue";

const message = ref("Hello from Vize!");
</script>

<template>
  <main class="app">
    <header class="app__header">
      <h1 class="app__title">{{ message }}</h1>
      <p class="app__subtitle">Compiled with the Vize Vite plugin ⚡️</p>
    </header>
    <ExampleCounter />
  </main>
</template>

<style scoped>
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2rem;
  padding: 2rem;
  font-family:
    system-ui,
    -apple-system,
    "Segoe UI",
    Roboto,
    sans-serif;
  color: #1b1f24;
  background: radial-gradient(120% 120% at 50% 0%, #effaf3 0%, #f7f9fb 55%);
}

.app__header {
  text-align: center;
}

.app__title {
  margin: 0;
  font-size: clamp(1.875rem, 4vw, 2.75rem);
  font-weight: 800;
  letter-spacing: -0.02em;
  color: #42b883;
}

.app__subtitle {
  margin: 0.5rem 0 0;
  font-size: 1rem;
  color: #6b7280;
}
</style>
