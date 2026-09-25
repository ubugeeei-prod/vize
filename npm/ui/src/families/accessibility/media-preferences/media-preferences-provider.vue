<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPreferencesContext } from "./media-preferences-context.ts";
import { mergeMediaPreferences, useMediaPreferencesTracker } from "./media-preferences-runtime.ts";
import type {
  ColorSchemePreference,
  ContrastPreference,
  MediaPreferences,
  MediaPreferencesOverrides,
  MediaPreferencesProviderExpose,
  MediaPreferencesSlotState,
} from "./media-preferences-types.ts";

const { initial = undefined, force = undefined } = defineProps<{
  /**
   * Preferences assumed on the server and until the client detects them, for
   * example from `Sec-CH-Prefers-Reduced-Motion` client hints or a cookie.
   *
   * @default undefined
   */
  readonly initial?: MediaPreferencesOverrides;

  /**
   * Preferences that always win over detection, for example an in-app
   * "reduce motion" setting.
   *
   * @default undefined
   */
  readonly force?: MediaPreferencesOverrides;
}>();

defineSlots<{
  /** Subtree that adapts to the effective preferences. */
  default(props: MediaPreferencesSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const tracker = useMediaPreferencesTracker({ initial: () => initial });
const preferences = computed<MediaPreferences>(() =>
  mergeMediaPreferences(tracker.preferences.value, force),
);

mediaPreferencesContext.provide(preferences);

type MediaPreferencesProviderSetupExpose = Omit<
  MediaPreferencesProviderExpose,
  "colorScheme" | "contrast" | "element" | "forcedColors" | "reducedMotion" | "reducedTransparency"
> & {
  readonly colorScheme: ComputedRef<ColorSchemePreference>;
  readonly contrast: ComputedRef<ContrastPreference>;
  readonly element: typeof element;
  readonly forcedColors: ComputedRef<boolean>;
  readonly reducedMotion: ComputedRef<boolean>;
  readonly reducedTransparency: ComputedRef<boolean>;
};

const exposed = {
  colorScheme: computed(() => preferences.value.colorScheme),
  contrast: computed(() => preferences.value.contrast),
  element,
  forcedColors: computed(() => preferences.value.forcedColors),
  reducedMotion: computed(() => preferences.value.reducedMotion),
  reducedTransparency: computed(() => preferences.value.reducedTransparency),
} satisfies MediaPreferencesProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="media-preferences-provider"
    part="root"
    :data-reduced-motion="preferences.reducedMotion ? 'true' : undefined"
    :data-reduced-transparency="preferences.reducedTransparency ? 'true' : undefined"
    :data-forced-colors="preferences.forcedColors ? 'true' : undefined"
    :data-prefers-contrast="preferences.contrast"
    :data-color-scheme="preferences.colorScheme"
  >
    <slot v-bind="preferences" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
