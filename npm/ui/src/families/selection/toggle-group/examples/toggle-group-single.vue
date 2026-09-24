<!-- Single-selection alignment toggle group with an uncontrolled default and a disabled item. -->
<script setup lang="ts">
import { ToggleGroup, ToggleGroupItem } from "../toggle-group.ts";
</script>

<template>
  <ToggleGroup type="single" default-value="left" aria-label="Text alignment">
    <ToggleGroupItem value="left">Left</ToggleGroupItem>
    <ToggleGroupItem value="center">Center</ToggleGroupItem>
    <ToggleGroupItem value="right">Right</ToggleGroupItem>
    <ToggleGroupItem value="justify" disabled>Justify</ToggleGroupItem>
  </ToggleGroup>
</template>
