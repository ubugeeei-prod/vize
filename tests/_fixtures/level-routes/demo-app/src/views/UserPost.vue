<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function backToProfile() {
  router.replace({ name: "user" });
}
</script>

<template>
  <nav>
    <button @click="backToProfile">profile</button>
    <RouterLink :to="{ name: 'home' }">home</RouterLink>
  </nav>
</template>
