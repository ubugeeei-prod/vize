<script setup lang="ts">
import { useRoute } from "vue-router";

const argument = useRoute("/about");
const generic = useRoute<"/about">();
void argument.params.userId;
void generic.params.userId;
</script>

<template>{{ argument.path }} {{ generic.path }}</template>
