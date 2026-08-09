<script setup lang="ts">
defineProps<{ value: string }>();
defineEmits<{ select: [value: string] }>();
defineSlots<{ default(props: { value: string }): unknown }>();
defineExpose<{ focus(): void }>();
defineModel<number>({ required: true });
</script>

<template>
  <button @click="$emit('select', value)">
    <slot :value="value" />
  </button>
</template>
