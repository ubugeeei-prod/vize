export const dialogRendererFixtures = [
  {
    filename: "DialogConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "./dialog.ts";
</script>

<template>
  <DialogRoot id="renderer-dialog">
    <DialogTrigger>Open dialog</DialogTrigger>
    <DialogPortal disabled>
      <DialogOverlay />
      <DialogContent>
        <DialogTitle>Renderer dialog</DialogTitle>
        <DialogDescription>Compiled across every renderer lane.</DialogDescription>
        <DialogClose>Close</DialogClose>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
`,
  },
] as const;
