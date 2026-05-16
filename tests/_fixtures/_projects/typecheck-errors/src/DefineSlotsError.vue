<script setup lang="ts">
const slots = defineSlots<{
  default(props: { message: string }): any;
}>();

const result: number = slots.default({ message: 123 });
console.log(result);
</script>

<template>
  <slot message="hello" />
</template>
