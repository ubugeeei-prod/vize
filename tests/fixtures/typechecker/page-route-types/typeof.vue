<script setup lang="ts">
import { useRoute } from "vue-router";

type CurrentRoute = ReturnType<typeof useRoute>;
function readId(route: CurrentRoute): number {
  return route.params.userId;
}
const id = readId(useRoute());
</script>

<template>{{ id.toFixed() }}</template>
