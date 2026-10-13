<script setup lang="ts">
const items = [{ id: "a" }];
</script>
<template>
  <div><button v-for="$attrs in items" v-bind="$attrs" /></div>
</template>
