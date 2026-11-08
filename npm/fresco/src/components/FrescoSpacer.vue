<script setup lang="ts">
import { Box } from "./Box.js";

defineOptions({ name: "Spacer" });
</script>

<template>
  <Box :flex-grow="1" :flex-shrink="1" />
</template>
