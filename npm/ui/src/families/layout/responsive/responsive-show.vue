<script setup lang="ts">
import { computed } from "vue";

import { defaultBreakpoints, useBreakpoint } from "./breakpoint.ts";
import type {
  BreakpointMap,
  ResponsiveHideMode,
  ResponsiveShowSlotState,
} from "./responsive-types.ts";

const {
  above = undefined,
  below = undefined,
  breakpoints = defaultBreakpoints,
  ssrWidth = undefined,
  hideMode = "unmount",
} = defineProps<{
  /**
   * Show only when the viewport is at least this breakpoint wide.
   *
   * @default undefined
   */
  readonly above?: string;

  /**
   * Show only when the viewport is narrower than this breakpoint.
   *
   * @default undefined
   */
  readonly below?: string;

  /**
   * Named minimum widths in CSS pixels.
   *
   * @default defaultBreakpoints
   */
  readonly breakpoints?: BreakpointMap;

  /**
   * Viewport width assumed during server rendering and hydration. Without it,
   * range-limited content is hidden until mount.
   *
   * @default undefined
   */
  readonly ssrWidth?: number;

  /**
   * `"unmount"` removes hidden content (the empty wrapper stays, `hidden`);
   * `"hidden"` keeps it in the DOM under the `hidden` wrapper (state is preserved).
   *
   * @default "unmount"
   */
  readonly hideMode?: ResponsiveHideMode;
}>();

defineSlots<{
  /** Content shown inside the breakpoint range. */
  default?(props: ResponsiveShowSlotState): unknown;
}>();

const breakpoint = useBreakpoint(() => breakpoints, { ssrWidth: () => ssrWidth });
const visible = computed(() => {
  if (above === undefined && below === undefined) return true;
  if (above !== undefined && !(above in breakpoints)) return false;
  if (below !== undefined && !(below in breakpoints)) return false;
  const aboveOk = above === undefined || breakpoint.isAbove(above);
  const belowOk = below === undefined || breakpoint.isBelow(below);
  return aboveOk && belowOk;
});

const wrapperStyle = { display: "contents" } as const;

defineExpose({ visible, width: breakpoint.width });
</script>

<template>
  <div
    data-vize-ui="responsive-show"
    :data-state="visible ? 'visible' : 'hidden'"
    :hidden="!visible"
    :style="wrapperStyle"
  >
    <slot v-if="visible || hideMode === 'hidden'" :visible="visible" />
  </div>
</template>

<style scoped>
/* Headless by design. display: contents keeps the wrapper out of layout. */
</style>
