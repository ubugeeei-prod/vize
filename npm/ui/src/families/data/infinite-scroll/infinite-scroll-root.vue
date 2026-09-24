<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { infiniteScrollContext } from "./infinite-scroll-context.ts";
import type { InfiniteScrollContextValue } from "./infinite-scroll-context.ts";
import type {
  InfiniteScrollLoader,
  InfiniteScrollObserverRoot,
  InfiniteScrollRootExpose,
  InfiniteScrollSlotState,
  InfiniteScrollState,
  InfiniteScrollTrigger,
} from "./infinite-scroll-types.ts";

const {
  id = undefined,
  hasMore = true,
  loader = undefined,
  loading = undefined,
  disabled = false,
  scrollRoot = "viewport",
  rootMargin = "256px",
  feed = false,
  total = undefined,
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
   * Whether more items can be requested. `false` settles the state to `complete`.
   *
   * @default true
   */
  readonly hasMore?: boolean;

  /**
   * Page loader called for every accepted request. A returned promise keeps the
   * state `loading` until it settles; a rejection moves to `error`.
   *
   * @default undefined
   */
  readonly loader?: InfiniteScrollLoader;

  /**
   * Controlled in-flight flag for consumers that load through the `loadMore` emit.
   * `undefined` derives the flag from {@link loader}.
   *
   * @default undefined
   */
  readonly loading?: boolean;

  /**
   * Suppress every request while keeping rendered items intact.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Intersection root: the viewport, or this root element as the scroll container.
   *
   * @default "viewport"
   */
  readonly scrollRoot?: InfiniteScrollObserverRoot;

  /**
   * Margin around the intersection root, so pages load before the sentinel is visible.
   *
   * @default "256px"
   */
  readonly rootMargin?: string;

  /**
   * Render the WAI-ARIA feed pattern: `role="feed"`, `aria-busy`, and PageUp/PageDown
   * focus movement between InfiniteScrollItem articles.
   *
   * @default false
   */
  readonly feed?: boolean;

  /**
   * Total item count announced as `aria-setsize`. `undefined` announces `-1` (unknown).
   *
   * @default undefined
   */
  readonly total?: number;

  /**
   * Accessible feed name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the feed.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired for every accepted request before {@link loader} runs. */
  loadMore: [trigger: InfiniteScrollTrigger];

  /** Fired after a {@link loader} promise rejects. */
  error: [reason: unknown, trigger: InfiniteScrollTrigger];

  /** Fired after every distinct loading-state transition. */
  stateChange: [state: InfiniteScrollState, previous: InfiniteScrollState];
}>();

defineSlots<{
  /** Items, sentinel, load-more control, and status. Receives the loading state. */
  default(props: InfiniteScrollSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "infinite-scroll" });
const pending = shallowRef(false);
const failed = shallowRef(false);
const failure = shallowRef<unknown>(undefined);
const refreshToken = shallowRef(0);
let generation = 0;

const busy = computed(() => loading ?? pending.value);
const state = computed<InfiniteScrollState>(() => {
  if (disabled) return "disabled";
  if (busy.value) return "loading";
  if (!hasMore) return "complete";
  return failed.value ? "error" : "idle";
});
const slotState = computed<InfiniteScrollSlotState>(() => ({
  busy: busy.value,
  error: failure.value,
  hasMore,
  state: state.value,
}));
const feedState = computed(() => feed);
const setSize = computed(() => total ?? -1);

watch(state, (next, previous) => {
  emit("stateChange", next, previous);
  if (next === "idle") refresh();
});

function refresh(): void {
  refreshToken.value += 1;
}

function isPromiseLike(value: unknown): value is PromiseLike<unknown> {
  return (
    (typeof value === "object" || typeof value === "function") &&
    value !== null &&
    "then" in value &&
    typeof value.then === "function"
  );
}

function settle(run: number, trigger: InfiniteScrollTrigger, rejected: boolean, reason: unknown) {
  if (run !== generation) return;
  pending.value = false;
  failed.value = rejected;
  failure.value = rejected ? reason : undefined;
  if (rejected) emit("error", reason, trigger);
}

function request(trigger: InfiniteScrollTrigger): boolean {
  if (state.value !== "idle" && !(state.value === "error" && trigger !== "sentinel")) return false;
  failed.value = false;
  failure.value = undefined;
  emit("loadMore", trigger);
  if (loader === undefined) return true;
  const run = ++generation;
  let result: unknown;
  try {
    result = loader(trigger);
  } catch (reason) {
    settle(run, trigger, true, reason);
    return true;
  }
  if (!isPromiseLike(result)) return true;
  pending.value = true;
  result.then(
    () => settle(run, trigger, false, undefined),
    (reason: unknown) => settle(run, trigger, true, reason),
  );
  return true;
}

function focusableItems(): HTMLElement[] {
  if (element.value === null) return [];
  return [
    ...element.value.querySelectorAll<HTMLElement>('[data-vize-ui="infinite-scroll-item"]'),
  ].filter((item) => item.closest('[data-vize-ui="infinite-scroll-root"]') === element.value);
}

function onKeydown(event: KeyboardEvent): void {
  if (!feed || (event.key !== "PageDown" && event.key !== "PageUp")) return;
  if (!(event.target instanceof Element)) return;
  const items = focusableItems();
  const current = event.target.closest<HTMLElement>('[data-vize-ui="infinite-scroll-item"]');
  const index = current === null ? -1 : items.indexOf(current);
  if (index < 0) return;
  const next = items[event.key === "PageDown" ? index + 1 : index - 1];
  event.preventDefault();
  next?.focus();
  if (next === undefined && event.key === "PageDown") request("api");
}

// Feed roots own PageUp/PageDown between focusable articles (WAI-ARIA feed pattern).
// The listener is attached on the client only, keeping server markup free of handlers.
onMounted(() => element.value?.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => element.value?.removeEventListener("keydown", onKeydown));

infiniteScrollContext.provide({
  element,
  feed: feedState,
  id: baseId,
  refreshToken,
  request,
  rootMargin: computed(() => rootMargin),
  scrollRoot: computed(() => scrollRoot),
  setSize,
  slotState,
  state,
} satisfies InfiniteScrollContextValue);

type InfiniteScrollRootSetupExpose = Omit<
  InfiniteScrollRootExpose,
  keyof InfiniteScrollSlotState | "element"
> & {
  readonly busy: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly error: typeof failure;
  readonly hasMore: ComputedRef<boolean>;
  readonly state: ComputedRef<InfiniteScrollState>;
};

const exposed = {
  busy,
  element,
  error: failure,
  hasMore: computed(() => hasMore),
  loadMore: () => request("api"),
  refresh,
  retry: () => state.value === "error" && request("api"),
  state,
} satisfies InfiniteScrollRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    ref="element"
    :role="feed ? 'feed' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-busy="busy ? 'true' : 'false'"
    data-vize-ui="infinite-scroll-root"
    part="root"
    :data-state="state"
    :data-scroll-root="scrollRoot"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
