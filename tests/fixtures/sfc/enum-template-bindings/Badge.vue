<script lang="ts">
export enum Tone {
  Info = "info",
  Warn = "warn",
}
enum LocalTone {
  Warn = "warn",
}
export const enum SharedMode {
  Warn = "warn",
}
const enum LocalMode {
  Warn = "warn",
}
</script>

<script setup lang="ts">
const enum SetupMode {
  Warn = "warn",
}
enum SetupTone {
  Warn = "warn",
}
const { tone = Tone.Info } = defineProps<{ tone?: Tone }>();
</script>

<template>
  <span
    :class="{
      warn: tone === Tone.Warn,
      local: tone === LocalTone.Warn,
      shared: tone === SharedMode.Warn,
      mode: tone === LocalMode.Warn,
      setup: tone === SetupMode.Warn,
      setupTone: tone === SetupTone.Warn,
    }"
  >{{ tone }}</span>
</template>
