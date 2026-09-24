<!-- Wrapping tag list laid out with a fixed gap and centered alignment. -->
<script setup lang="ts">
import { Cluster } from "../cluster.ts";

const tags = ["Vue", "TypeScript", "Accessibility", "SSR", "Headless UI", "Testing"];
</script>

<template>
  <Cluster as="ul" :gap="8" align="center" aria-label="Topics">
    <li v-for="tag in tags" :key="tag">{{ tag }}</li>
  </Cluster>
</template>
