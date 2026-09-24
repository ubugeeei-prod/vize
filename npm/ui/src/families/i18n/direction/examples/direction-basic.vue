<!-- Right-to-left subtree whose children read the inherited direction from DirectionProvider. -->
<script setup lang="ts">
import { DirectionProvider } from "../direction.ts";
</script>

<template>
  <DirectionProvider v-slot="{ dir }" dir="rtl" as="section">
    <p lang="ar">مرحبا بالعالم</p>
    <output>Direction: {{ dir }}</output>
  </DirectionProvider>
</template>
