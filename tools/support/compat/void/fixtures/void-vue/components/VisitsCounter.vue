<script setup lang="ts">
import { ref } from "vue";
import { fetch } from "void/client";

const visits = ref<number | null>(null);

async function refresh() {
  const body = await fetch("/api/visits");
  visits.value = body.visits;
}
</script>

<template>
  <button data-test="visits" @click="refresh">Visits: {{ visits ?? "?" }}</button>
</template>
