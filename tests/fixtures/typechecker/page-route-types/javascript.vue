<script setup>
import { useRoute } from "vue-router";

const route = useRoute();
const formatted = route.params.userId.toFixed();
</script>

<template>{{ formatted }} {{ $route.params.userId.toFixed() }}</template>
