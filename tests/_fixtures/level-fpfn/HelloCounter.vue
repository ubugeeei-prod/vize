<script setup>
import { ref } from "vue";

const count = ref(0);
const label = ref("clicks");

function increment() {
  count.value += 1;
}
</script>

<template>
  <button type="button" @click="increment">{{ label }}: {{ count }}</button>
</template>
