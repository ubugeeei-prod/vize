<!-- Informational note with a decorative icon, a title, a description, and a follow-up link. -->
<script setup lang="ts">
import { Callout } from "../callout.ts";
</script>

<template>
  <Callout tone="info">
    <template #icon>i</template>
    <template #title>Node.js 22 required</template>
    <template #description>
      The build tooling now relies on native TypeScript type stripping.
    </template>
    <template #actions>
      <a href="#upgrade-guide">Read the upgrade guide</a>
    </template>
  </Callout>
</template>
