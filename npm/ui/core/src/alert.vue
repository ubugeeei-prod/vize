<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  AlertExpose,
  AlertRole,
  AlertSlotState,
  AlertState,
  AlertVariant,
} from "./alert-types.ts";

const {
  id = undefined,
  role = "alert",
  variant = "info",
  open = true,
  atomic = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned root id for labels, descriptions, or anchors.
   *
   * @default undefined
   */
  readonly id?: string;

  /**
   * Live-region role: `alert` is assertive, `status` is polite.
   *
   * @default "alert"
   */
  readonly role?: AlertRole;

  /**
   * Styling variant mirrored to `data-variant`; no CSS is emitted.
   *
   * @default "info"
   */
  readonly variant?: AlertVariant;

  /**
   * Whether the alert is visible and announceable.
   *
   * @default true
   */
  readonly open?: boolean;

  /**
   * Whether assistive technology should present the whole region on updates.
   *
   * @default true
   */
  readonly atomic?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the alert.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the alert.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Renders alert content with current role, variant, and visibility state. */
  default(props: AlertSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const live = computed(() => (role === "alert" ? "assertive" : "polite"));
const state = computed<AlertState>(() => (open ? "open" : "closed"));

const exposed = {
  element,
} satisfies AlertExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id
    ref="element"
    :role
    :hidden="open ? undefined : true"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-live="live"
    :aria-atomic="atomic ? 'true' : 'false'"
    data-vize-ui="alert"
    :data-state="state"
    :data-variant="variant"
  >
    <slot :open :role :state="state" :variant />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
