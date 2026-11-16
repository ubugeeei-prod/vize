<script setup lang="ts">
defineSlots<{
  default(): unknown;
  title(props: { text: string }): unknown;
  "item-row"(props: { index: number }): unknown;
}>();
</script>

<template>
  <section><slot name="title" text="t" /><slot /><slot name="item-row" :index="0" /></section>
</template>
