<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "route-view-unknown" });
</script>

<template>
  <section>User</section>
</template>
