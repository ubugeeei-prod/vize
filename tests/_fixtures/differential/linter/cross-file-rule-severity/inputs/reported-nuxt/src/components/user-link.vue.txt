<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function open(id: number): void {
  router.push({ name: "users-id", params: { id: String(id) } });
}
</script>

<template>
  <button type="button" @click="open(1)">open</button>
</template>
