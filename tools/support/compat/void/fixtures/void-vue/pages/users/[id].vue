<script setup lang="ts">
import { useParams } from "@void/vue";
import type { Props } from "./[id].server.ts";

defineProps<Props>();
const params = useParams();
</script>

<template>
  <h1 data-test="user">{{ user.name }}</h1>
  <p data-test="param">{{ params.id }}</p>
</template>
