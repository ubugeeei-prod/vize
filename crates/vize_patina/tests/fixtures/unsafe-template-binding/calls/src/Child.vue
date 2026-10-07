<script setup lang="ts">
defineProps<{ label?: string; disabled?: boolean }>();
defineEmits<{ play: [] }>();
</script>

<template>
  <button type="button" :disabled="disabled">{{ label }}</button>
</template>
