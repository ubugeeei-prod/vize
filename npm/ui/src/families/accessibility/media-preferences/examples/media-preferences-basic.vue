<!-- App settings that override the operating system's reduced-motion preference. -->
<script setup lang="ts">
import { computed, ref, useId } from "vue";

import { MediaPreferencesProvider } from "../media-preferences.ts";

const reduceMotion = ref(false);
const checkboxId = useId();
const force = computed(() => (reduceMotion.value ? { reducedMotion: true } : {}));
</script>

<template>
  <MediaPreferencesProvider :force>
    <input :id="checkboxId" v-model="reduceMotion" type="checkbox" />
    <label :for="checkboxId">Reduce motion in this app</label>
  </MediaPreferencesProvider>
</template>
