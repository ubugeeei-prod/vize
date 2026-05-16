<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>

<template>
  <div>{{ count.toUpperCase() }}</div>
</template>
