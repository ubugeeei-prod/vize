<script setup lang="ts">
import { ref } from "vue";

function handleName(name: string): void {
  console.log(name);
}

const count = ref(0);
</script>

<template>
  <button @click="handleName">Click me</button>
  <span>{{ count }}</span>
</template>
