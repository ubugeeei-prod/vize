<script setup lang="ts">
import UserLink from "./components/UserLink.vue";
import "./navigation";
import type { PreviewState } from "./preview";
const state: PreviewState = { value: "live" };
</script>

<template>
  <main :data-state="state.value"><UserLink /></main>
</template>
