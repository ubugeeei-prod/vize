<script setup lang="ts">
import { createControllableState } from "../src/index.ts";

const props = defineProps<{
  readonly open?: boolean;
}>();

const emit = defineEmits<{
  change: [open: boolean];
}>();

const disclosure = createControllableState({
  value: () => props.open ?? false,
  onChange(next) {
    emit("change", next);
  },
});

const draft = createControllableState({
  defaultValue: "",
});

function toggle() {
  disclosure.set((open) => !open);
}
</script>

<template>
  <section>
    <button type="button" @click="toggle">
      {{ disclosure.value ? "Close" : "Open" }}
    </button>
    <input :value="draft.value" @input="draft.set(($event.target as HTMLInputElement).value)" />
  </section>
</template>
