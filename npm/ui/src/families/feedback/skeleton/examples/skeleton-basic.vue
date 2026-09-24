<!-- Labelled status skeleton that stands in for a profile card while it loads. -->
<script setup lang="ts">
import { Skeleton } from "../skeleton.ts";
</script>

<template>
  <Skeleton as="section" aria-label="Loading profile">
    <Skeleton block-size="1.25rem" inline-size="60%" />
    <Skeleton block-size="1rem" inline-size="80%" />
  </Skeleton>
</template>
