<script setup lang="ts">
import { onMounted, ref } from "vue";
const hydrated = ref(false);
onMounted(() => { hydrated.value = true; });
</script>

<template>
  <article class="card" :data-hydrated="hydrated">
    <h1>Styled before hydration</h1>
    <p>Scoped page CSS is part of the initial development SSR response.</p>
  </article>
</template>

<style scoped>
.card {
  box-sizing: border-box;
  width: 420px;
  padding: 24px;
  border: 3px solid rgb(40, 200, 90);
  background: rgb(18, 24, 30);
  color: rgb(240, 245, 250);
  font-family: sans-serif;
  font-size: 18px;
  line-height: 1.5;
}
h1 { margin: 0 0 16px; font-size: 24px; }
p { margin: 0; }
</style>
