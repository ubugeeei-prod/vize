<script setup>
import { ref, useCssModule } from "vue";
const emoji = "🐱";
const initial = 2;
const count = ref(initial);
const styles = useCssModule();
</script>
<template>
  <button id="regression-counter" class="counter" :class="styles.button" @click="count++">
    {{ emoji }} {{ count }}
  </button>
</template>
<style scoped>
.counter {
  color: rgb(12, 34, 56);
}
</style>
<style module>
.button {
  padding: 7px;
}
</style>
