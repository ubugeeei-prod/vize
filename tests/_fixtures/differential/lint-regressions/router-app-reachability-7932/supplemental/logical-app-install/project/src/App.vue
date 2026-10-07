<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();
router.push({ name: "user", params: { id: "1" } });
router.push({ name: "absent-destination" });
router.push({ name: "user" });
router.push({ name: "user", params: { id: "1", extra: "x" } });
router.push({ name: "user", params: { id: [] } });
</script>

<template>
  <main>Application</main>
</template>
