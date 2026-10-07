<script setup lang="ts">
defineProps<{ label: string }>();
defineEmits<{ click: [event: MouseEvent] }>();
</script>

<template>
  <button type="button" @click="$emit('click', $event)">{{ label }}</button>
</template>
