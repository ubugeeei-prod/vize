<!-- Status, count, and label badges annotating items in a pull request list. -->
<script setup lang="ts">
import { Badge } from "../badge.ts";
</script>

<template>
  <ul>
    <li>
      Fix hydration mismatch in tabs
      <Badge variant="status" tone="success">Merged</Badge>
    </li>
    <li>
      Add calendar range selection
      <Badge variant="status" tone="warning">Changes requested</Badge>
      <Badge variant="count" tone="info" aria-label="12 comments">12</Badge>
    </li>
    <li>
      Document carousel autoplay
      <Badge variant="label" tone="neutral">docs</Badge>
    </li>
  </ul>
</template>
