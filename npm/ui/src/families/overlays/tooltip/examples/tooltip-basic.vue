<!-- Tooltip that describes an icon-only trigger on hover and focus. -->
<script setup lang="ts">
import { TooltipContent, TooltipRoot, TooltipTrigger } from "../tooltip.ts";
</script>

<template>
  <TooltipRoot :delay-duration="300">
    <TooltipTrigger aria-label="Copy link">
      <span aria-hidden="true">&#x1F517;</span>
    </TooltipTrigger>
    <TooltipContent placement="top" :offset="6">Copy a shareable link to this page</TooltipContent>
  </TooltipRoot>
</template>
