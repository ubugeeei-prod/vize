<script setup lang="ts">
defineEmits<{
  (event: "submit", accepted: boolean): void;
}>();
</script>

<template>
  <button type="button">Submit</button>
</template>
