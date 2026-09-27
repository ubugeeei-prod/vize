<script setup lang="ts">
const optional = {} as { item?: string };
</script>
<template>
  <slot name="panel" v-bind="optional" />
  <slot name="panel" :item="1" />
</template>
