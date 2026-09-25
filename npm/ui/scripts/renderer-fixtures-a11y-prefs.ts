export const a11yPrefsRendererFixtures = [
  {
    filename: "FocusVisibleConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { FocusVisibleProvider, useFocusVisible } from "./families/accessibility/focus-visible/focus-visible.ts";

const focusVisible = useFocusVisible();
</script>

<template>
  <FocusVisibleProvider v-slot="{ modality }" attribute="data-ring">
    <button type="button" :data-outer-visible="focusVisible.isFocusVisible.value || undefined">
      Save ({{ modality ?? "none" }})
    </button>
  </FocusVisibleProvider>
</template>
`,
  },
  {
    filename: "MediaPreferencesConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  MediaPreferencesProvider,
  usePrefersReducedMotion,
} from "./families/accessibility/media-preferences/media-preferences.ts";

const reducedMotion = usePrefersReducedMotion();
</script>

<template>
  <MediaPreferencesProvider
    v-slot="{ colorScheme, contrast }"
    :initial="{ colorScheme: 'dark' }"
    :force="{ reducedMotion: reducedMotion }"
  >
    <p>{{ colorScheme }} / {{ contrast }}</p>
  </MediaPreferencesProvider>
</template>
`,
  },
] as const;
