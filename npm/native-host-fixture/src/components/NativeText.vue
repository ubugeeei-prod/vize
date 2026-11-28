<script setup lang="ts">
import type { NativeTextProps } from "../host-contract.js";

defineProps<NativeTextProps>();
</script>

<template>
  <native-text :text="text" />
</template>

<style scoped></style>
