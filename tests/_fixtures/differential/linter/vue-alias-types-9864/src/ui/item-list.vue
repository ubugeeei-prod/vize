<script setup lang="ts">
defineProps<{ items: string[] }>();
defineSlots<{ item(props: { value: string; index: number }): unknown }>();
</script>

<template>
  <ul>
    <li v-for="(value, index) in items" :key="value"><slot name="item" :value :index /></li>
  </ul>
</template>
