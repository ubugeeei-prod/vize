<script setup lang="ts">
const items = ["a", "b", "c"];
</script>

<template>
  <section>
    <ul>
      <li v-for="item in items" :key="item">{{ item }}</li>
    </ul>
  </section>
</template>
