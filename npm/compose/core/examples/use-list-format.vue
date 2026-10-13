<script setup lang="ts">
import { computed, ref } from "vue";
import { useListFormat } from "@vizejs/composable/use-list-format";

const locale = ref("en-US");
const speakers = ref(["Ada", "Grace", "Linus"]);
const options = computed(() => ({ locale: locale.value, type: "conjunction" as const }));
const { formatted } = useListFormat(speakers, options);
const next = ["Margaret", "Barbara", "Alan"];

function addSpeaker() {
  const name = next[speakers.value.length - 3];
  if (name) speakers.value = [...speakers.value, name];
}

function removeSpeaker() {
  speakers.value = speakers.value.slice(0, -1);
}
</script>

<template>
  <div class="composable-example">
    <p>Announce the speakers in a sentence that reads naturally in each language.</p>
    <label>
      Reader locale
      <select v-model="locale">
        <option value="en-US">English (United States)</option>
        <option value="de-DE">German (Germany)</option>
        <option value="ja-JP">Japanese (Japan)</option>
      </select>
    </label>
    <div class="example-actions">
      <button type="button" :disabled="speakers.length >= 6" @click="addSpeaker">
        Add speaker
      </button>
      <button type="button" :disabled="speakers.length === 0" @click="removeSpeaker">
        Remove speaker
      </button>
    </div>
    <output aria-live="polite">{{ speakers.length > 0 ? formatted : "No speakers yet." }}</output>
  </div>
</template>
