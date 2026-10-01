<script setup lang="ts">
defineProps<{ count: number; title: string; enabled?: boolean; object?: { id: string }; items: { label: string }[] }>();
</script>
<template>
  <p v-if="count">Items</p>
  <p v-else-if="title">Title</p>
  <p v-show="enabled">Enabled</p>
  <p v-if="object">Object</p>
  <p v-show="count > 0 && title !== ''">Explicit</p>
  <div v-for="item in items"><p v-if="item.label">Label</p></div>
</template>
