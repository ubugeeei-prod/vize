<!-- Image with a loading placeholder and a text fallback shown when the source fails. -->
<script setup lang="ts">
import { ImageContent, ImageFallback, ImagePlaceholder, ImageRoot } from "../image.ts";
</script>

<template>
  <ImageRoot src="/photos/harbor.jpg">
    <ImageContent alt="Fishing boats moored in the harbor at dusk" :width="800" :height="600" />
    <ImagePlaceholder>Loading photo…</ImagePlaceholder>
    <ImageFallback>Harbor photo unavailable</ImageFallback>
  </ImageRoot>
</template>
