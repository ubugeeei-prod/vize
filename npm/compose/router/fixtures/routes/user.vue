<template>
  <main data-route="user">User {{ id }}</main>
</template>

<script setup lang="ts">
defineOptions({ name: "RouterUserFixture" });

const id = "fixture-user";
</script>
