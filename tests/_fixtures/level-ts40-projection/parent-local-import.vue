<script setup lang="ts">
import ChildLocal from "./child-local-import.vue"

const parentLabel = "parent"
</script>
<template><ChildLocal :message="parentLabel" /></template>
