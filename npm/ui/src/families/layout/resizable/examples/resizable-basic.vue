<!-- Notes panel with a controlled size that can be resized from its right edge and bottom-right corner. -->
<script setup lang="ts">
import { ref } from "vue";

import { Resizable, ResizableHandle } from "../resizable.ts";
import type { ResizableSize } from "../resizable.ts";

const size = ref<ResizableSize>({ width: 320, height: 200 });
</script>

<template>
  <div>
    <Resizable
      v-model:size="size"
      :min-width="200"
      :max-width="560"
      :min-height="120"
      :max-height="400"
      :step="10"
    >
      <p>Meeting notes: drag a handle or focus it and use the arrow keys.</p>
      <ResizableHandle edge="e" aria-label="Resize notes width" />
      <ResizableHandle edge="se" aria-label="Resize notes" />
    </Resizable>
    <output>{{ size.width }} × {{ size.height }} px</output>
  </div>
</template>
