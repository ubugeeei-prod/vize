<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useSpeechRecognition } from "@vizejs/composable/use-speech-recognition";

const mounted = useMounted();
const { supported, listening, result, interimResult, toggle } = useSpeechRecognition({
  lang: "en-US",
  interimResults: true,
});
</script>

<template>
  <div class="composable-example">
    <p>Dictate a search query instead of typing it.</p>
    <button type="button" :disabled="!mounted || !supported" @click="() => toggle()">
      {{ listening ? "Stop listening" : "Start dictation" }}
    </button>
    <p>Heard: {{ result || interimResult || "…" }}</p>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? listening
            ? "Listening."
            : "Ready to listen."
          : "Speech recognition is not available in this browser."
    }}</output>
  </div>
</template>
