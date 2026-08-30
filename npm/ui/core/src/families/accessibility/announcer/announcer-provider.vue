<script setup lang="ts">
import { toRef } from "vue";

import { useAnnouncerOwnership } from "./announcer-runtime.ts";
import type { AnnouncerController, AnnouncerPoliteness } from "./announcer-types.ts";

const { politeness = "polite", atomic = true } = defineProps<{
  /**
   * Default urgency for announcements that do not name one.
   *
   * @default "polite"
   */
  readonly politeness?: AnnouncerPoliteness;

  /**
   * Whether assistive technology should present a whole channel on each update.
   *
   * @default true
   */
  readonly atomic?: boolean;
}>();

defineSlots<{
  /** Subtree served by this announcer. Receives the owning controller. */
  default(props: { readonly announcer: AnnouncerController }): unknown;
}>();

const ownership = useAnnouncerOwnership({ politeness: toRef(() => politeness) });

defineExpose({
  announcer: ownership.announcer,
  isOwner: ownership.isOwner,
  announce: ownership.announcer.announce,
  clear: ownership.announcer.clear,
});
</script>

<template>
  <div data-vize-ui="announcer" :data-vize-announcer="ownership.isOwner ? 'owner' : 'delegate'">
    <div
      v-if="ownership.isOwner"
      data-vize-ui="announcer-region"
      aria-live="polite"
      role="status"
      :aria-atomic="atomic ? 'true' : 'false'"
    >
      {{ ownership.announcer.politeMessage.value }}
    </div>
    <div
      v-if="ownership.isOwner"
      data-vize-ui="announcer-region"
      aria-live="assertive"
      role="alert"
      :aria-atomic="atomic ? 'true' : 'false'"
    >
      {{ ownership.announcer.assertiveMessage.value }}
    </div>
    <slot :announcer="ownership.announcer" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
