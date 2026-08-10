<script setup lang="ts">
const model = defineModel<number>({ required: true });
void model;
</script>

<template>
  <input v-model="model" />
</template>
