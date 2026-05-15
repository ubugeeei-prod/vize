<script setup lang="ts" vapor>
import { ref } from "vue";

const message: string = "Hello from Vapor script setup";
const count = ref(0);
</script>

<template>
  <div class="setup">
    <p>{{ message }}</p>
    <span>{{ count }}</span>
  </div>
</template>
