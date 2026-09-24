<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { marqueeContext } from "./marquee-context.ts";
import type { MarqueeContextValue } from "./marquee-context.ts";
import {
  marqueeCopies,
  marqueeDuration,
  marqueeOrientation,
  marqueeStyle,
} from "./marquee-geometry.ts";
import { resolveMarqueeMessages } from "./marquee-types.ts";
import type {
  MarqueeDirection,
  MarqueeMeasurement,
  MarqueeMessageOverrides,
  MarqueeMessages,
  MarqueeOrientation,
  MarqueePauseReason,
  MarqueeRootExpose,
  MarqueeSlotState,
  MarqueeState,
} from "./marquee-types.ts";

const {
  id = undefined,
  direction = "left",
  speed = 50,
  repeat = "auto",
  playing = undefined,
  defaultPlaying = true,
  pauseOnHover = true,
  pauseOnFocus = true,
  respectReducedMotion = true,
  messages = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Consumer-owned root id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Direction the content travels.
   *
   * @default "left"
   */
  readonly direction?: MarqueeDirection;

  /**
   * Travel speed in CSS pixels per second; the loop duration is derived from the
   * measured copy length, so speed stays constant whatever the content size.
   *
   * @default 50
   */
  readonly speed?: number;

  /**
   * Number of rendered copies, or `"auto"` to fill the viewport seamlessly.
   *
   * @default "auto"
   */
  readonly repeat?: number | "auto";

  /**
   * Controlled play intent (`v-model:playing`). `undefined` selects {@link defaultPlaying}.
   *
   * @default undefined
   */
  readonly playing?: boolean;

  /**
   * Initial play intent for uncontrolled use.
   *
   * @default true
   */
  readonly defaultPlaying?: boolean;

  /**
   * Pause while a mouse or pen pointer rests on the marquee.
   *
   * @default true
   */
  readonly pauseOnHover?: boolean;

  /**
   * Pause while keyboard focus is inside the marquee.
   *
   * @default true
   */
  readonly pauseOnFocus?: boolean;

  /**
   * Stay paused while `prefers-reduced-motion: reduce` matches until the user presses play.
   *
   * @default true
   */
  readonly respectReducedMotion?: boolean;

  /**
   * Localized strings for the pause button.
   *
   * @default undefined
   */
  readonly messages?: MarqueeMessageOverrides | undefined;

  /**
   * Accessible name of the marquee region.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the marquee region.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired when the play intent requests a new controlled value. */
  "update:playing": [playing: boolean];

  /** Fired after every distinct animation-state transition. */
  stateChange: [state: MarqueeState, reason: MarqueePauseReason | null];
}>();

defineSlots<{
  /** Content, pause button, and decorations. Receives the animation state. */
  default(props: MarqueeSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "marquee" });
const intent = useControllableState<boolean>({
  value: () => playing,
  defaultValue: () => defaultPlaying,
});
const hovered = shallowRef(false);
const focused = shallowRef(false);
const reducedMotion = shallowRef(false);
const motionConsent = shallowRef(false);
const measurement = shallowRef<MarqueeMeasurement | null>(null);
let reducedMotionQuery: MediaQueryList | null = null;

const directionState = computed<MarqueeDirection>(() => direction);
const orientation = computed<MarqueeOrientation>(() => marqueeOrientation(direction));
const resolvedMessages = computed<MarqueeMessages>(() => resolveMarqueeMessages(messages));
const reducedMotionBlocks = computed(
  () => respectReducedMotion && reducedMotion.value && !motionConsent.value,
);
const pauseReason = computed<MarqueePauseReason | null>(() => {
  if (!intent.value.value) return "user";
  if (reducedMotionBlocks.value) return "reduced-motion";
  if (pauseOnFocus && focused.value) return "focus";
  if (pauseOnHover && hovered.value) return "hover";
  return null;
});
const state = computed<MarqueeState>(() => (pauseReason.value === null ? "running" : "paused"));
const effectivePlaying = computed(() => intent.value.value && !reducedMotionBlocks.value);
const distance = computed<number | null>(() => measurement.value?.distance ?? null);
const copies = computed<number>(() =>
  marqueeCopies(repeat, distance.value, measurement.value?.viewport ?? null),
);
const duration = computed<number | null>(() => marqueeDuration(distance.value, speed));
const style = computed<string>(() => marqueeStyle(distance.value, duration.value, copies.value));
const slotState = computed<MarqueeSlotState>(() => ({
  copies: copies.value,
  direction,
  duration: duration.value,
  pauseReason: pauseReason.value,
  playing: intent.value.value,
  state: state.value,
}));

watch(state, (next) => emit("stateChange", next, pauseReason.value));

function setPlaying(next: boolean): boolean {
  if (next) motionConsent.value = true;
  if (!intent.set(next)) return false;
  emit("update:playing", next);
  return true;
}

function play(): boolean {
  const consented = !motionConsent.value && reducedMotionBlocks.value;
  return setPlaying(true) || consented;
}

function toggle(): boolean {
  return effectivePlaying.value ? setPlaying(false) : play();
}

function setMeasurement(next: MarqueeMeasurement): void {
  if (
    measurement.value?.distance === next.distance &&
    measurement.value.viewport === next.viewport
  ) {
    return;
  }
  measurement.value = next;
}

function onPointerEnter(event: PointerEvent): void {
  if (event.pointerType !== "touch") hovered.value = true;
}

function onPointerLeave(): void {
  hovered.value = false;
}

function onFocusIn(): void {
  focused.value = true;
}

function onFocusOut(event: FocusEvent): void {
  if (event.relatedTarget instanceof Node && element.value?.contains(event.relatedTarget)) return;
  focused.value = false;
}

function onReducedMotionChange(event: MediaQueryListEvent): void {
  reducedMotion.value = event.matches;
}

// Listeners and media queries attach on the client only, so server markup and
// the hydration render agree.
onMounted(() => {
  element.value?.addEventListener("pointerenter", onPointerEnter);
  element.value?.addEventListener("pointerleave", onPointerLeave);
  element.value?.addEventListener("focusin", onFocusIn);
  element.value?.addEventListener("focusout", onFocusOut);
  if (typeof globalThis.matchMedia === "function") {
    reducedMotionQuery = globalThis.matchMedia("(prefers-reduced-motion: reduce)");
    reducedMotion.value = reducedMotionQuery.matches;
    reducedMotionQuery.addEventListener("change", onReducedMotionChange);
  }
});

onBeforeUnmount(() => {
  element.value?.removeEventListener("pointerenter", onPointerEnter);
  element.value?.removeEventListener("pointerleave", onPointerLeave);
  element.value?.removeEventListener("focusin", onFocusIn);
  element.value?.removeEventListener("focusout", onFocusOut);
  reducedMotionQuery?.removeEventListener("change", onReducedMotionChange);
  reducedMotionQuery = null;
});

marqueeContext.provide({
  copies,
  direction: directionState,
  effectivePlaying,
  element,
  id: baseId,
  messages: resolvedMessages,
  orientation,
  setMeasurement,
  slotState,
  state,
  toggle,
} satisfies MarqueeContextValue);

type MarqueeRootSetupExpose = Omit<MarqueeRootExpose, keyof MarqueeSlotState | "element"> & {
  readonly copies: ComputedRef<number>;
  readonly direction: ComputedRef<MarqueeDirection>;
  readonly duration: ComputedRef<number | null>;
  readonly element: typeof element;
  readonly pauseReason: ComputedRef<MarqueePauseReason | null>;
  readonly playing: ComputedRef<boolean>;
  readonly state: ComputedRef<MarqueeState>;
};

const exposed = {
  copies,
  direction: directionState,
  duration,
  element,
  pause: () => setPlaying(false),
  pauseReason,
  play,
  playing: computed(() => intent.value.value),
  state,
  toggle,
} satisfies MarqueeRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    role="marquee"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :style
    data-vize-ui="marquee-root"
    part="root"
    :data-state="state"
    :data-pause-reason="pauseReason ?? undefined"
    :data-direction="directionState"
    :data-orientation="orientation"
    :data-measured="duration === null ? 'false' : 'true'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Animate the track with consumer CSS, for example:
   [data-vize-ui="marquee-root"] { overflow: hidden; }
   [data-vize-ui="marquee-track"] { display: flex; width: max-content;
     animation: vize-marquee var(--vize-ui-marquee-duration, 0s) linear infinite; }
   [data-vize-ui="marquee-root"][data-state="paused"] [data-vize-ui="marquee-track"] {
     animation-play-state: paused; }
   [data-direction="right"] [data-vize-ui="marquee-track"] { animation-direction: reverse; }
   @keyframes vize-marquee { to { translate: calc(-1 * var(--vize-ui-marquee-distance, 0px)); } } */
</style>
