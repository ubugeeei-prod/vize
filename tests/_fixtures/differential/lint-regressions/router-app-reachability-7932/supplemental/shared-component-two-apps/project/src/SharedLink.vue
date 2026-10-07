<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "alpha-home", params: { id: "1" } });
router.push({ name: "beta-home", params: { id: "2" } });
router.push({ name: "alpha-home", params: { id: [] } });
router.push({ name: "beta-home", params: { id: "2", extra: "x" } });
</script>

<template>
  <span>Shared link</span>
</template>
