<script setup lang="ts">
import { ref } from "vue";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useSpeechSynthesis } from "@vizejs/composable/use-speech-synthesis";

const mounted = useMounted();
const text = ref("Welcome to Vue Fes Japan.");
const { supported, status, speak, cancel } = useSpeechSynthesis(text, { lang: "en-US", rate: 1 });
</script>

<template>
  <div class="composable-example">
    <p>Read an announcement aloud with the system voice.</p>
    <label>
      Announcement
      <input v-model="text" />
    </label>
    <div class="example-actions">
      <button type="button" :disabled="!mounted || !supported" @click="() => speak()">
        Read aloud
      </button>
      <button type="button" :disabled="!mounted || !supported" @click="() => cancel()">Stop</button>
    </div>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : supported
          ? `Speech: ${status}`
          : "Speech synthesis is not available in this browser."
    }}</output>
  </div>
</template>
