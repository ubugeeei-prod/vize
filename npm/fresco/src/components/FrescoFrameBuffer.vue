<script setup lang="ts">
import type { FrameBufferProps } from "./frame-buffer-types.js";

defineProps<FrameBufferProps>();

defineOptions({ name: "FrameBuffer" });

const rawElement = "raw";
</script>

<template>
  <component
    :is="rawElement"
    :lines
    :style
    :fg
    :bg
    :bold
    :dim
    :italic
    :underline
    :inverse
    :border
  />
</template>
