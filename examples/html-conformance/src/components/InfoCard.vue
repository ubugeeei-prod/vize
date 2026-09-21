<script setup lang="ts">
defineProps<{ title: string }>();
</script>

<template>
  <div class="card">
    <strong>{{ title }}</strong>
  </div>
</template>
