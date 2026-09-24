<script setup lang="ts">
import PopoverContent from "../../overlays/popover/popover-content.vue";
import type { PopoverPlacement } from "../../overlays/popover/popover-types.ts";
import { hotspotMarkerContext } from "./hotspot-context.ts";
import type { HotspotMarkerSlotState } from "./hotspot-types.ts";

const {
  placement = "top",
  offset = 8,
  portalDisabled = false,
  closeOnEscape = true,
  closeOnPointerDownOutside = true,
  closeOnFocusOutside = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Preferred placement relative to the marker before collision handling.
   *
   * @default "top"
   */
  readonly placement?: PopoverPlacement;

  /**
   * Gap between marker and content in CSS pixels.
   *
   * @default 8
   */
  readonly offset?: number;

  /**
   * Render in place instead of portalling to `document.body`.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;

  /**
   * Close when Escape is pressed while the content or marker has focus.
   *
   * @default true
   */
  readonly closeOnEscape?: boolean;

  /**
   * Close on pointer down outside the content and marker.
   *
   * @default true
   */
  readonly closeOnPointerDownOutside?: boolean;

  /**
   * Close when focus moves outside. Off by default so neighbouring markers, and
   * focus restored from another marker's content, never close this one.
   *
   * @default false
   */
  readonly closeOnFocusOutside?: boolean;

  /**
   * Accessible dialog name when no labelled element is referenced.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the content.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Hotspot details. Receives the marker state. */
  default(props: HotspotMarkerSlotState): unknown;
}>();

const marker = hotspotMarkerContext.use();
</script>

<template>
  <PopoverContent
    :placement
    :offset
    :portal-disabled
    :close-on-escape
    :close-on-pointer-down-outside
    :close-on-focus-outside
    :aria-label
    :aria-labelledby
    data-hotspot-content=""
    :data-hotspot-id="marker.id.value"
  >
    <slot
      :id="marker.id.value"
      :open="marker.open.value"
      :disabled="marker.disabled.value"
      :state="marker.disabled.value ? 'disabled' : marker.open.value ? 'open' : 'closed'"
    />
  </PopoverContent>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
