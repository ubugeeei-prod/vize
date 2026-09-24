<script setup lang="ts">
import { computed, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useVisibilityObserver } from "../../interaction/measure/measure-runtime.ts";
import { imageContext } from "./image-context.ts";
import type { ImageContextValue } from "./image-context.ts";
import { resolveImageCandidates } from "./image-source.ts";
import type {
  ImageRootExpose,
  ImageSlotState,
  ImageSource,
  ImageStatus,
  ImageStatusChangeReason,
} from "./image-types.ts";

const {
  src = undefined,
  defer = false,
  rootMargin = "200px",
  allowInsecure = false,
} = defineProps<{
  /**
   * Image source or ordered candidate chain. Each failed candidate advances to the
   * next safe one; unsafe or malformed candidates are skipped and never rendered.
   *
   * @default undefined
   */
  readonly src?: ImageSource | null;

  /**
   * Wait until the root intersects the viewport before attaching any source.
   * Complements native `loading="lazy"` for background-critical or custom scroll roots.
   *
   * @default false
   */
  readonly defer?: boolean;

  /**
   * Intersection margin used while {@link defer} waits for visibility.
   *
   * @default "200px"
   */
  readonly rootMargin?: string;

  /**
   * Permit unencrypted `http:` candidates for local development.
   *
   * @default false
   */
  readonly allowInsecure?: boolean;
}>();

const emit = defineEmits<{
  /** Fired after the attached candidate loads. `nativeEvent` is `null` for pre-hydration loads. */
  load: [nativeEvent: Event | null, src: string];

  /** Fired after one candidate fails, before the next candidate is attached. */
  error: [nativeEvent: Event | null, src: string];

  /** Fired after every distinct status transition with the cause of the transition. */
  statusChange: [status: ImageStatus, previous: ImageStatus, reason: ImageStatusChangeReason];
}>();

defineSlots<{
  /** Compound Image parts. Receives the current loading lifecycle state. */
  default(props: ImageSlotState): unknown;
}>();

const element = useTemplateRef<HTMLSpanElement>("element");
const candidates = computed(() => resolveImageCandidates(src, { allowInsecure }));
const index = shallowRef(0);
const loaded = shallowRef(false);
const visible = shallowRef(!defer);
let pendingReason: ImageStatusChangeReason = "source";

const status = computed<ImageStatus>(() => {
  if (index.value >= candidates.value.length) return "error";
  if (!visible.value) return "idle";
  return loaded.value ? "loaded" : "loading";
});
const attached = computed(() => status.value === "loading" || status.value === "loaded");
const currentSrc = computed(() => (attached.value ? candidates.value[index.value] : undefined));
const candidateIndex = computed(() => (attached.value ? index.value : -1));
const candidateCount = computed(() => candidates.value.length);
const slotState = computed<ImageSlotState>(() => ({
  candidateCount: candidateCount.value,
  candidateIndex: candidateIndex.value,
  src: currentSrc.value,
  status: status.value,
}));

function sameCandidates(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((value, position) => value === right[position]);
}

watch(candidates, (next, previous) => {
  if (sameCandidates(next, previous)) return;
  pendingReason = "source";
  index.value = 0;
  loaded.value = false;
});

watch(status, (next, previous) => {
  emit("statusChange", next, previous, pendingReason);
});

function markVisible(): void {
  if (visible.value) return;
  pendingReason = "visible";
  visible.value = true;
  observer.disconnect();
}

watch(
  () => defer,
  (next) => {
    if (!next) markVisible();
  },
);

const observer = useVisibilityObserver({
  get rootMargin() {
    return rootMargin;
  },
  onVisibilityChange(entries) {
    if (entries.some((entry) => entry.isIntersecting)) markVisible();
  },
});

onMounted(() => {
  if (visible.value) return;
  if (!observer.isSupported || element.value === null) {
    markVisible();
    return;
  }
  observer.observe(element.value);
});

function handleLoad(nativeEvent: Event | null): void {
  if (status.value !== "loading" || currentSrc.value === undefined) return;
  emit("load", nativeEvent, currentSrc.value);
  pendingReason = "load";
  loaded.value = true;
}

function handleError(nativeEvent: Event | null): void {
  if (currentSrc.value === undefined) return;
  emit("error", nativeEvent, currentSrc.value);
  pendingReason = "error";
  loaded.value = false;
  index.value += 1;
}

function retry(): boolean {
  if (candidates.value.length === 0) return false;
  pendingReason = "retry";
  index.value = 0;
  loaded.value = false;
  return true;
}

imageContext.provide({
  candidateIndex,
  handleError,
  handleLoad,
  slotState,
  src: currentSrc,
  status,
} satisfies ImageContextValue);

type ImageRootSetupExpose = Omit<ImageRootExpose, keyof ImageSlotState | "element"> & {
  readonly candidateCount: ComputedRef<number>;
  readonly candidateIndex: ComputedRef<number>;
  readonly element: typeof element;
  readonly src: ComputedRef<string | undefined>;
  readonly status: ComputedRef<ImageStatus>;
};

const exposed = {
  candidateCount,
  candidateIndex,
  element,
  retry,
  src: currentSrc,
  status,
} satisfies ImageRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    ref="element"
    data-vize-ui="image-root"
    part="root"
    :data-status="status"
    :data-deferred="defer ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
