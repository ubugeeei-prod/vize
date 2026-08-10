<script setup lang="ts">
defineSlots<{
  default(props: { value: "slot" }): unknown;
}>();
</script>

<template>
  <slot value="slot" />
</template>
