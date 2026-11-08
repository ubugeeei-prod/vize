<script setup lang="ts">
import type { FrameBufferProps } from "./frame-buffer-types.js";

defineProps<FrameBufferProps>();

defineOptions({ name: "FrameBuffer" });
</script>

<template>
  <raw
    :lines="lines"
    :style="style"
    :fg="fg"
    :bg="bg"
    :bold="bold"
    :dim="dim"
    :italic="italic"
    :underline="underline"
    :inverse="inverse"
    :border="border"
  />
</template>
