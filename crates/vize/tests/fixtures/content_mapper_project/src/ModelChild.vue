<script setup lang="ts">
const title = defineModel<string>("title", { required: true });
void title;
</script>

<template>
  <input v-model="title" />
</template>
