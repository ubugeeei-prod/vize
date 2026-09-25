<!-- Inline text tokens used for a paragraph with emphasized and muted fragments. -->
<script setup lang="ts">
import { Text } from "../text.ts";
</script>

<template>
  <p>
    <Text weight="semibold">Deployment finished.</Text>
    <Text tone="success" size="sm">All checks passed</Text>
    <Text as="small" tone="muted" size="xs">in 42 seconds</Text>
  </p>
</template>
