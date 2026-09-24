<!-- Signature pad bound with v-model, with a sign-here guide and undo, redo, and clear buttons. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  SignaturePad,
  SignaturePadCanvas,
  SignaturePadClear,
  SignaturePadGuide,
  SignaturePadRedo,
  SignaturePadUndo,
  type SignatureValue,
} from "../signature-pad.ts";

const signature = ref<SignatureValue>([]);
</script>

<template>
  <SignaturePad v-slot="{ empty }" v-model="signature" name="signature" :width="400" :height="160">
    <SignaturePadCanvas aria-label="Your signature" />
    <SignaturePadGuide>Sign above the line</SignaturePadGuide>
    <SignaturePadUndo>Undo</SignaturePadUndo>
    <SignaturePadRedo>Redo</SignaturePadRedo>
    <SignaturePadClear>Clear</SignaturePadClear>
    <p>{{ empty ? "Not signed yet" : "Signed" }}</p>
  </SignaturePad>
</template>
