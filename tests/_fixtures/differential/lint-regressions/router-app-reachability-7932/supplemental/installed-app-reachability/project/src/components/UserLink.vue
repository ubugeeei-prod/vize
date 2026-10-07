<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "live-user", params: { id: "1" } });
router.push({ name: "live-user", params: { id: [] } });
</script>

<template>
  <span>User link</span>
</template>
