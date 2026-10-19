<script setup lang="ts">
import { Link } from "@void/vue";
</script>

<template>
  <nav>
    <Link href="/">Home</Link>
    <Link href="/users">Users</Link>
    <Link href="/stats">Stats</Link>
  </nav>
  <main><slot /></main>
</template>
