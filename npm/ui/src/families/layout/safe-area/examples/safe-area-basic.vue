<!-- Footer toolbar padded past the device home indicator using the bottom safe-area inset. -->
<script setup lang="ts">
import { SafeArea } from "../safe-area.ts";
import type { SafeAreaEdge } from "../safe-area.ts";

const edges: readonly SafeAreaEdge[] = ["bottom", "left", "right"];
</script>

<template>
  <SafeArea v-slot="{ insets }" as="footer" :edges apply="padding">
    <button type="button">Reply</button>
    <button type="button">Forward</button>
    <small>Bottom inset: {{ insets.bottom }}px</small>
  </SafeArea>
</template>
