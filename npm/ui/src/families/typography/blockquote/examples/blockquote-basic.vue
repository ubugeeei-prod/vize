<!-- Attributed quotation with a cite URL and a visible figure caption. -->
<script setup lang="ts">
import { Blockquote } from "../blockquote.ts";
</script>

<template>
  <figure>
    <Blockquote cite="https://www.w3.org/standards/webdesign/accessibility" size="lg">
      <p>
        The power of the Web is in its universality. Access by everyone regardless of disability is
        an essential aspect.
      </p>
    </Blockquote>
    <figcaption>Tim Berners-Lee, W3C Director</figcaption>
  </figure>
</template>
