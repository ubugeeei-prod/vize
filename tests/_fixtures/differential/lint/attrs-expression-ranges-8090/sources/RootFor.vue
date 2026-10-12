<script setup lang="ts">
const items = [{}];
</script>
<template>
  <div v-for="item in items" v-bind="$attrs" />
</template>
