<script setup lang="ts">
import type { NativeViewProps } from "../host-contract.js";

withDefaults(defineProps<NativeViewProps>(), { gap: 0, padding: 0 });
defineSlots<{ default(): unknown }>();
</script>

<template>
  <native-view :gap="gap" :padding="padding"><slot /></native-view>
</template>

<style scoped></style>
