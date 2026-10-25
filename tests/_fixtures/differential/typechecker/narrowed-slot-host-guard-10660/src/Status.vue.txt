<script setup lang="ts">
import { ref } from "vue";
import Box from "./Box.vue";

const status = ref<"pending" | "error" | "success">("pending");
const fruits = ref(["Apple"]);
</script>

<template>
  <p v-if="status === 'pending'">Loading</p>
  <div v-else>
    <Box v-for="fruit in fruits" :key="fruit">
      <template #default>{{ fruit }}</template>
    </Box>
  </div>
</template>
