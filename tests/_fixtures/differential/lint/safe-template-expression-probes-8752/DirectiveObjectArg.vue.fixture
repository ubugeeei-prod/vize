<script setup lang="ts">
const isA = (o: { kind: string }): boolean => o.kind === "a";
const isPositive = (n: number): boolean => n > 0;
</script>

<template>
  <p v-if="isA({ kind: 'a' })">reported</p>
  <p v-if="isPositive(1)">not reported</p>
  <p>{{ isA({ kind: "a" }) }}</p>
</template>
