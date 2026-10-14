<script setup lang="ts">
import { ref } from "vue";
import { useLanguageDetector } from "@vizejs/composable/use-builtin-ai";
import { useMounted } from "@vizejs/composable/use-mounted";

const mounted = useMounted();
const detector = useLanguageDetector({ expectedInputLanguages: ["en", "ja", "fr"] });
const comment = ref("この発表はとても良かったです");
const language = ref<string>();

async function detect() {
  const [best] = await detector.detect(comment.value);
  language.value = best
    ? `${best.detectedLanguage} (${Math.round(best.confidence * 100)}%)`
    : "unknown";
}
</script>

<template>
  <div class="composable-example">
    <p>Detect the language of a comment on the device, without sending it to a server.</p>
    <label>
      Comment
      <input v-model="comment" />
    </label>
    <button type="button" :disabled="!mounted || !detector.supported.value" @click="detect">
      Detect language
    </button>
    <output aria-live="polite">{{
      !mounted
        ? "Checking browser support…"
        : !detector.supported.value
          ? "Built-in AI is not available in this browser; detect the language on your server instead."
          : (language ?? `Model status: ${detector.status.value}`)
    }}</output>
  </div>
</template>
