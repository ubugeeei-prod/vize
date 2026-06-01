<script setup lang="ts">
defineProps<{
  title?: string;
}>();
</script>

<template>
  <article>
    <slot>{{ title ?? "Panel" }}</slot>
  </article>
</template>
