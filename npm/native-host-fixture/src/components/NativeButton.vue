<script setup lang="ts">
import type { NativeButtonProps } from "../host-contract.js";

defineProps<NativeButtonProps>();
const emit = defineEmits<{ press: [] }>();
</script>

<template>
  <native-button :label="label" @press="emit('press')" />
</template>

<style scoped></style>
