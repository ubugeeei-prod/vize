<script setup lang="ts">
import { computed, ref } from "vue";

import { useInteractionHooks } from "./interaction-hooks.ts";

const activations = ref(0);
const interactions = useInteractionHooks({
  focusWithin: {},
  press: {
    onPress() {
      activations.value += 1;
    },
  },
});

const focused = computed<boolean | undefined>(() => interactions.isFocused.value || undefined);
const focusWithin = computed<boolean | undefined>(
  () => interactions.isFocusWithin.value || undefined,
);
const hovered = computed<boolean | undefined>(() => interactions.isHovered.value || undefined);
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
    :data-pressed="pressed"
  >
    Activated {{ activations }} times
  </button>
</template>

<style scoped>
button {
  font: inherit;
}
</style>
