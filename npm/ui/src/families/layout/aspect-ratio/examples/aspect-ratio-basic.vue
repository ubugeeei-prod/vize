<!-- Widescreen 16:9 frame that keeps a video poster at a stable ratio while it loads. -->
<script setup lang="ts">
import { AspectRatio } from "../aspect-ratio.ts";
</script>

<template>
  <figure>
    <AspectRatio :ratio="16 / 9">
      <img
        src="https://example.com/media/keynote-poster.jpg"
        alt="Keynote stage before the opening talk"
        width="1600"
        height="900"
      />
    </AspectRatio>
    <figcaption>Opening keynote</figcaption>
  </figure>
</template>
