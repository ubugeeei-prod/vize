<!-- Meaningful SVG icon named by its title next to a decorative icon hidden from assistive technology. -->
<script setup lang="ts">
import { Icon } from "../icon.ts";
</script>

<template>
  <p>
    <Icon title="Warning" size="sm">
      <path d="M12 3 2 21h20L12 3Z" />
      <path d="M12 10v5" />
      <path d="M12 18h.01" />
    </Icon>
    Your trial ends in three days.
  </p>
  <p>
    <Icon aria-hidden>
      <path d="M5 12h14" />
      <path d="m13 6 6 6-6 6" />
    </Icon>
    Continue to billing
  </p>
</template>
