<script setup lang="ts">
defineProps<{ count: number; label?: string }>();
defineEmits<{ change: [value: number]; "update:label": [value: string] }>();
defineSlots<{ default(): unknown; footer(props: { total: number }): unknown }>();
</script>

<template>
  <div><slot /><slot name="footer" :total="1" /></div>
</template>
