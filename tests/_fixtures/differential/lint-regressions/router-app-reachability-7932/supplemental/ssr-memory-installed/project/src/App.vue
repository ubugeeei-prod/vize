<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "ssr-user", params: { id: "1" } });
router.push({ name: "absent-destination" });
</script>

<template>
  <main>Server application</main>
</template>
