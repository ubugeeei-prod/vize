<script setup lang="ts">
defineProps<{ title: string; featured?: boolean }>();
</script>

<template>
  <article class="badge-card" :data-card="title">
    <strong>{{ title }}</strong>
    <span v-if="featured" class="featured">Featured</span>
    <slot />
  </article>
</template>

<style scoped>
.badge-card {
  color: rebeccapurple;
}
</style>
