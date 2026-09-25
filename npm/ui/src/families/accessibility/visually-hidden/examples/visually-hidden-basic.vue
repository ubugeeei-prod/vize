<!-- Icon-only button whose accessible name is supplied by visually hidden text. -->
<script setup lang="ts">
import { VisuallyHidden } from "../visually-hidden.ts";
</script>

<template>
  <button type="button">
    <span aria-hidden="true">&times;</span>
    <VisuallyHidden>Close dialog</VisuallyHidden>
  </button>
</template>
