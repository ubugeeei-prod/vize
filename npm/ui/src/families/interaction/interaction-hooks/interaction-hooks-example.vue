<script setup lang="ts">
import { computed, onScopeDispose, ref } from "vue";

import { useInteractionHooks } from "./interaction-hooks.ts";

const activations = ref(0);
const shortcutActivations = ref(0);
const interactions = useInteractionHooks({
  modality: {},
  focusWithin: {},
  shortcuts: {
    platform: "standard",
  },
  press: {
    onPress() {
      activations.value += 1;
    },
  },
});

const releaseShortcut = interactions.shortcutController?.register({
  shortcut: "Mod+K",
  description: "Increment shortcut activation count",
  handler() {
    shortcutActivations.value += 1;
  },
});
if (releaseShortcut) onScopeDispose(releaseShortcut);

const focused = computed<boolean | undefined>(() => interactions.isFocused.value || undefined);
const focusWithin = computed<boolean | undefined>(
  () => interactions.isFocusWithin.value || undefined,
);
const hovered = computed<boolean | undefined>(() => interactions.isHovered.value || undefined);
const modality = computed<string | undefined>(
  () => interactions.currentModality.value ?? undefined,
);
const pressed = computed<boolean | undefined>(() => interactions.isPressed.value || undefined);
</script>

<template>
  <button
    v-bind="interactions.interactionProps"
    data-vize-ui="interaction-hooks-example"
    type="button"
    :data-focused="focused"
    :data-focus-within="focusWithin"
    :data-hovered="hovered"
    :data-modality="modality"
    :data-pressed="pressed"
  >
    Activated {{ activations }} times
    <span>Shortcut {{ shortcutActivations }}</span>
  </button>
</template>

<style scoped>
button {
  font: inherit;
}
</style>
