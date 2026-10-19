<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useDropZone } from "@vizejs/composable/use-drop-zone";

const zone = useTemplateRef<HTMLElement>("zone");
const { isOverDropZone, files } = useDropZone(zone, { accept: ["image/*"], multiple: true });
</script>

<template>
  <div class="composable-example">
    <p>Accept images dropped onto an area and list them.</p>
    <div
      ref="zone"
      role="region"
      aria-label="Image drop area"
      :style="{
        border: '2px dashed',
        padding: '16px',
        background: isOverDropZone ? '#42b88333' : 'transparent',
      }"
    >
      {{ isOverDropZone ? "Release to add the images" : "Drop images here" }}
    </div>
    <output aria-live="polite">{{
      files?.length ? `Added: ${files.map((file) => file.name).join(", ")}` : "No images yet."
    }}</output>
  </div>
</template>
