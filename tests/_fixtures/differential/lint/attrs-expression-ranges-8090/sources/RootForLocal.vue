<script setup lang="ts">
const items = [{}];
</script>
<template>
  <div v-for="$attrs in items" v-bind="$attrs" />
</template>
