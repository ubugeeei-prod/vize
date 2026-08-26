<script setup lang="ts">
import { reactive, watchEffect } from "vue";

import {
  localeContext,
  resolveDirection,
  type DirectionPreference,
  type TextDirection,
} from "./locale-runtime.ts";

const { locale = "en-US", direction = "ltr" } = defineProps<{
  /**
   * BCP 47 locale for the subtree.
   *
   * @default "en-US"
   */
  readonly locale?: string;

  /**
   * Writing direction. `auto` resolves from the locale when possible.
   *
   * @default "ltr"
   */
  readonly direction?: DirectionPreference;
}>();

defineSlots<{
  /** Localized subtree. Receives the resolved locale and direction. */
  default(props: { readonly locale: string; readonly direction: TextDirection }): unknown;
}>();

const value = reactive({
  locale: "en-US",
  direction: "ltr" as TextDirection,
});

watchEffect(() => {
  value.locale = locale;
  value.direction = resolveDirection(direction, locale);
});

localeContext.provide(value);
</script>

<template>
  <div data-vize-ui="locale" :lang="value.locale" :dir="value.direction">
    <slot :locale="value.locale" :direction="value.direction" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
