<script setup lang="ts" generic="T">
defineProps<{ value: T }>();
defineSlots<{ label(props: { value: T }): unknown }>();
</script>

<template>
  <span><slot name="label" :value="value" /></span>
</template>
