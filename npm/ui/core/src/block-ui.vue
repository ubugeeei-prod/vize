<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type {
  BlockUIAnnouncement,
  BlockUIElement,
  BlockUIExpose,
  BlockUIInteraction,
  BlockUIReason,
  BlockUISlotState,
  BlockUIState,
} from "./block-ui-types.ts";
import type { PrimitiveAs } from "./primitive.ts";

const {
  as = "section",
  blocked = false,
  reason = "loading",
  interaction = "none",
  announce = "off",
  label = undefined,
} = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "section"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether the region is currently blocked by in-progress or unavailable work.
   *
   * @default false
   */
  readonly blocked?: boolean;

  /**
   * Consumer styling and status reason mirrored to `data-reason`.
   *
   * @default "loading"
   */
  readonly reason?: BlockUIReason;

  /**
   * Optional interaction policy. `inert` applies the native inert attribute only while blocked.
   *
   * @default "none"
   */
  readonly interaction?: BlockUIInteraction;

  /**
   * Optional live-region announcement policy used only when `label` is non-empty.
   *
   * @default "off"
   */
  readonly announce?: BlockUIAnnouncement;

  /**
   * Accessible announcement label used when `announce` is not `off`.
   *
   * @default undefined
   */
  readonly label?: string;

  /**
   * Consumed so BlockUI state owns `aria-busy` instead of fallthrough attrs.
   *
   * @default undefined
   */
  readonly "aria-busy"?: string | boolean;

  /**
   * Consumed so BlockUI state owns the native inert policy instead of fallthrough attrs.
   *
   * @default undefined
   */
  readonly inert?: boolean | "";
}>();

defineSlots<{
  /** Renders blocked-region content with the current blocking and announcement state. */
  default(props: BlockUISlotState): unknown;
}>();

const element = useTemplateRef<BlockUIElement>("element");
const blockedState = computed(() => blocked);
const state = computed<BlockUIState>(() => (blockedState.value ? "blocked" : "idle"));
const reasonState = computed(() => reason);
const interactionState = computed(() => interaction);
const announcementState = computed(() => announce);
const announcementLabel = computed(() =>
  label != null && label.trim().length > 0 ? label : undefined,
);
const announcementRole = computed<"alert" | "status" | undefined>(() => {
  if (announcementLabel.value === undefined) return undefined;
  if (announcementState.value === "polite") return "status";
  if (announcementState.value === "assertive") return "alert";
  return undefined;
});
const announcementAriaLive = computed<"assertive" | "polite" | undefined>(() =>
  announcementRole.value === undefined
    ? undefined
    : (announcementState.value as "assertive" | "polite"),
);
const busyAttribute = computed(() => (blockedState.value ? "true" : undefined));
const inertAttribute = computed(() =>
  blockedState.value && interactionState.value === "inert" ? true : undefined,
);
const slotState = computed<BlockUISlotState>(() => ({
  announcement: announcementState.value,
  blocked: blockedState.value,
  interaction: interactionState.value,
  reason: reasonState.value,
  state: state.value,
}));

type BlockUISetupExpose = Omit<
  BlockUIExpose,
  "announcement" | "blocked" | "element" | "interaction" | "reason" | "state"
> & {
  readonly announcement: typeof announcementState;
  readonly blocked: typeof blockedState;
  readonly element: typeof element;
  readonly interaction: typeof interactionState;
  readonly reason: typeof reasonState;
  readonly state: typeof state;
};

const exposed = {
  announcement: announcementState,
  blocked: blockedState,
  element,
  interaction: interactionState,
  reason: reasonState,
  state,
} satisfies BlockUISetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    part="root"
    data-vize-ui="block-ui"
    :data-state="state"
    :data-reason="reasonState"
    :data-interaction="interactionState"
    :data-announcement="announcementState"
    :aria-busy="busyAttribute"
    :inert="inertAttribute"
    :role="announcementRole"
    :aria-live="announcementAriaLive"
    :aria-label="announcementLabel"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
