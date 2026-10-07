<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "beta-home", params: { id: "1" } });
router.push({ name: "alpha-home", params: { id: "1" } });
</script>

<template>
  <main>Beta</main>
</template>
