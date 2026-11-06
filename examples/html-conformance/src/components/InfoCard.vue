<script setup lang="ts">
const { title } = defineProps<{ title: string }>();
</script>

<template>
  <div class="card">
    <strong>{{ title }}</strong>
  </div>
</template>
