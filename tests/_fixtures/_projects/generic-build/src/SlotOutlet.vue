<script setup lang="ts">
const fallbackLabel = "No records yet";
</script>

<template>
  <section>
    <header>
      <slot name="title">Generic Title</slot>
    </header>
    <slot />
    <slot name="empty" :label="fallbackLabel">
      <p>{{ fallbackLabel }}</p>
    </slot>
  </section>
</template>
