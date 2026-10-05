<script setup lang="ts">
import { useRoute } from "vue-router";

const argument: number = useRoute("/users/[userId=int]").params.userId;
const generic: number = useRoute<"/users/[userId=int]">().params.userId;
definePage<"src/pages/users/[userId=int].vue">({ params: { path: { userId: "int" } } });
</script>

<template>{{ argument.toFixed() }} {{ generic.toFixed() }}</template>
