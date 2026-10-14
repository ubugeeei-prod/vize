<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useShare } from "@vizejs/composable/use-share";

const mounted = useMounted();
const { supported, share } = useShare({
  title: "Vize",
  text: "A Vue toolchain in Rust",
  url: "https://vizejs.dev",
});
</script>

<template>
  <div class="composable-example">
    <p>Use the system share sheet on devices that have one, and a copyable link elsewhere.</p>
    <button v-if="mounted && supported" type="button" @click="() => share()">Share Vize</button>
    <p v-else>Copy this link: https://vizejs.dev</p>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? "The share sheet is available."
          : "No share sheet here; showing a link instead."
    }}</output>
  </div>
</template>
