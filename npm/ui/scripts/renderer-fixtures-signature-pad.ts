export const signaturePadRendererFixtures = [
  {
    filename: "SignaturePadConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  SignaturePadCanvas,
  SignaturePadClear,
  SignaturePadGuide,
  SignaturePadRedo,
  SignaturePadRoot,
  SignaturePadUndo,
} from "./families/media/signature-pad/signature-pad.ts";
import type { SignatureValue } from "./families/media/signature-pad/signature-pad.ts";

const strokes = ref<SignatureValue>([]);
</script>

<template>
  <SignaturePadRoot v-model="strokes" name="signature" :width="480" :height="160">
    <template #default="{ empty }">
      <SignaturePadGuide>Sign above the line</SignaturePadGuide>
      <SignaturePadCanvas aria-label="Your signature" aria-describedby="signature-help">
        <line x1="16" y1="140" x2="464" y2="140" stroke="currentColor" />
      </SignaturePadCanvas>
      <p id="signature-help">Draw with a mouse, pen, or finger, or type your name instead.</p>
      <SignaturePadUndo>Undo</SignaturePadUndo>
      <SignaturePadRedo>Redo</SignaturePadRedo>
      <SignaturePadClear>{{ empty ? "Nothing to clear" : "Clear" }}</SignaturePadClear>
    </template>
  </SignaturePadRoot>
</template>
`,
  },
] as const;
