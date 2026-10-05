<script setup lang="ts">
import { useRoute as aliasedRoute } from "vue-router";

const api = { useRoute: () => 42, definePage: (value: number) => value };
const member: number = api.useRoute();
const memberPage: number = api.definePage(3);
function local() {
  const useRoute = () => 42;
  const definePage = (value: number) => value;
  return useRoute() + definePage(3);
}
const text = "useRoute() and definePage({})";
// useRoute(); definePage({})
const allRoutes = aliasedRoute();
</script>

<template>{{ member }} {{ memberPage }} {{ local() }} {{ text }} {{ allRoutes.path }}</template>
