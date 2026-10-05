<script setup lang="ts">
import { useRoute } from "vue-router";

const route = useRoute();
void route.params.userId;
</script>

<template>{{ route.path }}</template>
