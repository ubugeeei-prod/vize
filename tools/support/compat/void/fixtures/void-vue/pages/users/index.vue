<script setup lang="ts">
import { Link } from "@void/vue";
import type { Props } from "./index.server.ts";

defineProps<Props>();
</script>

<template>
  <ul>
    <li v-for="user in users" :key="user.id">
      <Link :href="`/users/${user.id}`">{{ user.name }}</Link>
    </li>
  </ul>
</template>
