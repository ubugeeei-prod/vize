<!-- Semantic headings whose visual size is decoupled from their document level. -->
<script setup lang="ts">
import { Heading } from "../heading.ts";
</script>

<template>
  <article>
    <Heading :level="1" size="2xl" weight="bold">Release notes</Heading>
    <Heading :level="2" size="lg">Version 2.4</Heading>
    <p>Faster builds and a redesigned settings page.</p>
    <Heading :level="3" size="sm" tone="muted">Breaking changes</Heading>
    <p>The legacy config format is no longer read.</p>
  </article>
</template>
