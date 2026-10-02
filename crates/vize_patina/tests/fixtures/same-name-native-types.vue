<script setup lang="ts">
const { ariaLabel = undefined } = defineProps<{ readonly ariaLabel?: string }>();
const ariaLabelledby: string = "label";
const ariaDescription: any = "unsafe";
</script>

<template>
  <div :aria-label :aria-labelledby :aria-description></div>
</template>
