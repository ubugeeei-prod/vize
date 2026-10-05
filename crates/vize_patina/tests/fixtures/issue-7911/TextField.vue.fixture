<script setup lang="ts">
const props = defineProps<{ id?: string | number; label?: string }>();
const elements: (HTMLElement | null)[] = [];
</script>

<template>
  <div>
    <input :id="props.id as string | undefined" type="text" />
    <input :aria-label="props.label satisfies string | undefined" type="text" />
    <input :title="String(props.id as string | number)" type="text" />
    <input :ref="(el) => elements.push(el as HTMLElement | null)" type="text" />
  </div>
</template>
