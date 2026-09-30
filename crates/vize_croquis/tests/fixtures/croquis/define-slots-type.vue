<script setup lang="ts">
const slots = defineSlots<{
  default(props: { message: string }): any;
  header(props: { title: string }): any;
}>();
console.log(slots);
</script>

<template>
  <div>
    <slot name="header" title="Hello" />
    <slot message="World" />
  </div>
</template>
