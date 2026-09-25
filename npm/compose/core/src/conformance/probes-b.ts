import type { ComposableProbeMap } from "./probe-types.ts";

/** Conformance probes for catalog entries in group B (test-only). */
export const probesGroupB: ComposableProbeMap = {
  "./network": {
    kind: "component",
    setup: `import { useNetwork } from "@/network.ts";
const network = useNetwork({ ssrOnline: false });`,
    state:
      "{ supported: network.isSupported.value, online: network.isOnline.value, saveData: network.saveData.value }",
    server: { supported: false, online: false, saveData: false },
    client: { supported: false, online: true, saveData: false },
  },
  "./on-click-outside": {
    kind: "component",
    setup: `import { ref, useTemplateRef } from "vue";
import { onClickOutside } from "@/on-click-outside.ts";
const target = useTemplateRef<HTMLElement>("target");
const outside = ref(0);
onClickOutside(target, () => {
  outside.value += 1;
});`,
    state: "{ outside }",
    markup: '<div ref="target">inside</div>',
    server: { outside: 0 },
    client: { outside: 0 },
  },
  "./on-key-stroke": {
    kind: "component",
    setup: `import { ref } from "vue";
import { onKeyStroke } from "@/on-key-stroke.ts";
const presses = ref(0);
onKeyStroke("Enter", () => {
  presses.value += 1;
});`,
    state: "{ presses }",
    server: { presses: 0 },
    client: { presses: 0 },
  },
  "./on-long-press": {
    kind: "component",
    setup: `import { ref, useTemplateRef } from "vue";
import { onLongPress } from "@/on-long-press.ts";
const target = useTemplateRef<HTMLElement>("target");
const pressed = ref(false);
onLongPress(target, () => {
  pressed.value = true;
});`,
    state: "{ pressed }",
    markup: '<button ref="target" type="button">hold</button>',
    server: { pressed: false },
    client: { pressed: false },
  },
  "./on-start-typing": {
    kind: "component",
    setup: `import { ref } from "vue";
import { isTypingKeystroke, onStartTyping } from "@/on-start-typing.ts";
const typed = ref(0);
onStartTyping(() => {
  typed.value += 1;
});
const guard = typeof isTypingKeystroke;`,
    state: "{ typed, guard }",
    server: { typed: 0, guard: "function" },
    client: { typed: 0, guard: "function" },
  },
  "./page-leave": {
    kind: "component",
    setup: `import { usePageLeave } from "@/page-leave.ts";
const left = usePageLeave();`,
    state: "{ left }",
    server: { left: false },
    client: { left: false },
  },
  "./parent-element": {
    kind: "component",
    setup: `import { computed, useTemplateRef } from "vue";
import { useParentElement } from "@/parent-element.ts";
const child = useTemplateRef<HTMLElement>("child");
const parent = useParentElement(child);
const hasParent = computed(() => parent.value !== null);`,
    state: "{ hasParent }",
    markup: '<span ref="child">child</span>',
    server: { hasParent: false },
    client: { hasParent: true },
  },
  "./pinch": {
    kind: "component",
    setup: `import { useTemplateRef } from "vue";
import { usePinch } from "@/pinch.ts";
const target = useTemplateRef<HTMLElement>("target");
const pinch = usePinch(target);`,
    state: "{ pinching: pinch.isPinching.value, scale: pinch.scale.value }",
    markup: '<div ref="target">pinch</div>',
    server: { pinching: false, scale: 1 },
    client: { pinching: false, scale: 1 },
  },
  "./pointer": {
    kind: "component",
    setup: `import { toPointerKind, usePointer } from "@/pointer.ts";
const pointer = usePointer();
const kind = toPointerKind("mouse");`,
    state: "{ x: pointer.state.x, y: pointer.state.y, inside: pointer.isInside.value, kind }",
    server: { x: 0, y: 0, inside: false, kind: "mouse" },
    client: { x: 0, y: 0, inside: false, kind: "mouse" },
  },
  "./pointer-lock": {
    kind: "component",
    setup: `import { usePointerLock } from "@/pointer-lock.ts";
// happy-dom lacks the Pointer Lock API: a client-only fake host proves activation.
class FakeLockHost extends EventTarget {
  pointerLockElement: Element | null = null;
  exitPointerLock(): void {}
}
const host = typeof window === "undefined" ? undefined : new FakeLockHost();
const lock = usePointerLock(undefined, { host: () => host });`,
    state: "{ supported: lock.isSupported.value, locked: lock.isLocked.value }",
    server: { supported: false, locked: false },
    client: { supported: true, locked: false },
  },
  "./preferences": {
    kind: "component",
    setup: `import {
  usePreferredColorScheme,
  usePreferredContrast,
  usePreferredDark,
  usePreferredReducedTransparency,
} from "@/preferences.ts";
const dark = usePreferredDark();
const scheme = usePreferredColorScheme();
const contrast = usePreferredContrast();
const transparency = usePreferredReducedTransparency();`,
    state: "{ dark, scheme, contrast, transparency }",
    server: {
      dark: false,
      scheme: "no-preference",
      contrast: "no-preference",
      transparency: false,
    },
    client: { dark: false, scheme: "light", contrast: "no-preference", transparency: false },
  },
  "./preferred-languages": {
    kind: "component",
    setup: `import { usePreferredLanguages } from "@/preferred-languages.ts";
const languages = usePreferredLanguages({ ssrLanguages: ["ja"] });`,
    state: "{ languages }",
    server: { languages: ["ja"] },
    client: { languages: ["en-US", "en"] },
  },
  "./reactify": {
    kind: "component",
    setup: `import { ref } from "vue";
import { reactify } from "@/reactify.ts";
const add = reactify((left: number, right: number) => left + right);
const left = ref(2);
const sum = add(left, 3);`,
    state: "{ sum }",
    server: { sum: 5 },
    client: { sum: 5 },
  },
  "./ref-auto-reset": {
    kind: "component",
    setup: `import { refAutoReset } from "@/ref-auto-reset.ts";
const message = refAutoReset("idle", 1000);`,
    state: "{ message }",
    server: { message: "idle" },
    client: { message: "idle" },
  },
  "./ref-debounced": {
    kind: "component",
    setup: `import { ref } from "vue";
import { refDebounced } from "@/ref-debounced.ts";
const source = ref("typed");
const debounced = refDebounced(source, 100);`,
    state: "{ debounced }",
    server: { debounced: "typed" },
    client: { debounced: "typed" },
  },
  "./ref-default": {
    kind: "component",
    setup: `import { ref } from "vue";
import { refDefault } from "@/ref-default.ts";
const raw = ref<string | undefined>(undefined);
const value = refDefault(raw, "fallback");`,
    state: "{ value }",
    server: { value: "fallback" },
    client: { value: "fallback" },
  },
  "./resize-observer": {
    kind: "component",
    setup: `import { useTemplateRef } from "vue";
import { useElementSize, useResizeObserver } from "@/resize-observer.ts";
const target = useTemplateRef<HTMLElement>("target");
const observer = useResizeObserver(target, () => undefined);
const size = useElementSize(target, { initialSize: { width: 10, height: 20 } });`,
    state:
      "{ supported: observer.isSupported.value, width: size.width.value, height: size.height.value }",
    markup: '<div ref="target">box</div>',
    server: { supported: false, width: 10, height: 20 },
    client: { supported: true, width: 10, height: 20 },
  },
  "./retry-async": {
    kind: "component",
    setup: `import { ref } from "vue";
import { retryAsync } from "@/retry-async.ts";
const attempts = ref(0);
const kind = typeof retryAsync;`,
    state: "{ attempts, kind }",
    server: { attempts: 0, kind: "function" },
    client: { attempts: 0, kind: "function" },
  },
  "./retry-delay": {
    kind: "component",
    setup: `import { calculateRetryDelay } from "@/retry-delay.ts";
const delay = calculateRetryDelay(2, { initialDelayMs: 100, jitterRatio: 0 });`,
    state: "{ delay }",
    server: { delay: 200 },
    client: { delay: 200 },
  },
  "./scope": {
    kind: "component",
    setup: `import { tryOnScopeDispose } from "@/scope.ts";
const registered = tryOnScopeDispose(() => undefined);`,
    state: "{ registered }",
    server: { registered: true },
    client: { registered: true },
  },
  "./screen-orientation": {
    kind: "component",
    setup: `import { useScreenOrientation } from "@/screen-orientation.ts";
// A client-only fake host whose orientation differs from the server fallback.
class FakeOrientation extends EventTarget {
  readonly type = "portrait-primary";
  readonly angle = 90;
}
const host =
  typeof window === "undefined" ? undefined : { screen: { orientation: new FakeOrientation() } };
const screen = useScreenOrientation({ ssrOrientation: "landscape-primary", host: () => host });`,
    state:
      "{ supported: screen.isSupported.value, orientation: screen.orientation.value, angle: screen.angle.value }",
    server: { supported: false, orientation: "landscape-primary", angle: 0 },
    client: { supported: true, orientation: "portrait-primary", angle: 90 },
  },
  "./scroll": {
    kind: "component",
    setup: `import { useTemplateRef } from "vue";
import { useScroll, useWindowScroll } from "@/scroll.ts";
const target = useTemplateRef<HTMLElement>("target");
const scroll = useScroll(target);
// Scroll the client window before hydration: the server renders 0.
if (typeof window !== "undefined") window.scrollTo(0, 40);
const windowScroll = useWindowScroll();`,
    state:
      "{ x: scroll.x.value, y: scroll.y.value, wx: windowScroll.x.value, wy: windowScroll.y.value }",
    markup: '<div ref="target">scroll</div>',
    server: { x: 0, y: 0, wx: 0, wy: 0 },
    client: { x: 0, y: 0, wx: 0, wy: 40 },
  },
  "./scroll-lock": {
    kind: "component",
    setup: `import { useTemplateRef } from "vue";
import { useScrollLock } from "@/scroll-lock.ts";
const target = useTemplateRef<HTMLElement>("target");
const locked = useScrollLock(target);`,
    state: "{ locked }",
    markup: '<div ref="target">lock</div>',
    server: { locked: false },
    client: { locked: false },
  },
  "./standard-schema": {
    kind: "component",
    setup: `import { formatSchemaPath, isStandardSchema } from "@/standard-schema.ts";
const schema = { "~standard": { version: 1, vendor: "probe", validate: (value: unknown) => ({ value }) } };
const valid = isStandardSchema(schema);
const path = formatSchemaPath(["items", { key: 0 }]);`,
    state: "{ valid, path }",
    server: { valid: true, path: "items.0" },
    client: { valid: true, path: "items.0" },
  },
  "./swipe": {
    kind: "component",
    setup: `import { useTemplateRef } from "vue";
import { usePointerSwipe, useSwipe } from "@/swipe.ts";
const target = useTemplateRef<HTMLElement>("target");
const swipe = useSwipe(target);
const pointerSwipe = usePointerSwipe(target);`,
    state:
      "{ swiping: swipe.isSwiping.value, direction: swipe.direction.value, pointer: pointerSwipe.isSwiping.value }",
    markup: '<div ref="target">swipe</div>',
    server: { swiping: false, direction: "none", pointer: false },
    client: { swiping: false, direction: "none", pointer: false },
  },
  "./sync-ref": {
    kind: "component",
    setup: `import { ref } from "vue";
import { syncRef } from "@/sync-ref.ts";
const left = ref("a");
const right = ref("b");
syncRef(left, right);`,
    state: "{ left, right }",
    server: { left: "a", right: "a" },
    client: { left: "a", right: "a" },
  },
  "./temporal": {
    kind: "component",
    setup: `import { Temporal, useTemporalNow, useTemporalZonedDateTime } from "@/temporal.ts";
const fixed = Temporal.Instant.fromEpochMilliseconds(0);
const clock = useTemporalNow({ now: () => fixed, paused: true });
const zoned = useTemporalZonedDateTime({ now: () => fixed, paused: true, timeZone: "UTC" });`,
    state: "{ instant: clock.instant.value.toString(), zoned: zoned.toString() }",
    server: { instant: "1970-01-01T00:00:00Z", zoned: "1970-01-01T00:00:00+00:00[UTC]" },
    client: { instant: "1970-01-01T00:00:00Z", zoned: "1970-01-01T00:00:00+00:00[UTC]" },
  },
  "./text-selection": {
    kind: "component",
    setup: `import { useTextSelection } from "@/text-selection.ts";
const selection = useTextSelection();`,
    state: "{ text: selection.text.value, ranges: selection.ranges.value.length }",
    server: { text: "", ranges: 0 },
    client: { text: "", ranges: 0 },
  },
  "./textarea-autosize": {
    kind: "component",
    setup: `import { ref, useTemplateRef } from "vue";
import { useTextareaAutosize } from "@/textarea-autosize.ts";
const target = useTemplateRef<HTMLTextAreaElement>("target");
const autosize = useTextareaAutosize(target, { input: ref("hello") });`,
    state: "{ input: autosize.input.value }",
    markup: '<textarea ref="target"></textarea>',
    server: { input: "hello" },
    client: { input: "hello" },
  },
  "./timeout-scheduler": {
    kind: "exempt",
    reason: "Declares the TimeoutScheduler type only; the entry has no runtime exports to mount.",
  },
  "./to-reactive": {
    kind: "component",
    setup: `import { ref } from "vue";
import { toReactive } from "@/to-reactive.ts";
const source = ref({ count: 1 });
const state = toReactive(source);`,
    state: "{ count: state.count }",
    server: { count: 1 },
    client: { count: 1 },
  },
  "./transition": {
    kind: "component",
    setup: `import { ref } from "vue";
import { TransitionPresets, cubicBezier, useTransition } from "@/transition.ts";
const source = ref(10);
const output = useTransition(source, { duration: 100 });
const eased = cubicBezier([0, 0, 1, 1])(0.5);
const presets = Object.keys(TransitionPresets).length > 0;`,
    state: "{ output, eased, presets }",
    server: { output: 10, eased: 0.5, presets: true },
    client: { output: 10, eased: 0.5, presets: true },
  },
  "./until": {
    kind: "component",
    setup: `import { ref } from "vue";
import { until } from "@/until.ts";
const ready = ref(true);
const settled = ref(false);
void until(ready).toBe(true).then(() => {
  settled.value = true;
});`,
    state: "{ ready }",
    server: { ready: true },
    client: { ready: true },
  },
  "./use-array": {
    kind: "component",
    setup: `import { ref } from "vue";
import {
  useArrayEvery,
  useArrayFilter,
  useArrayFind,
  useArrayMap,
  useArrayReduce,
  useArraySome,
  useArrayUnique,
} from "@/use-array.ts";
const list = ref([1, 2, 2, 3]);
const even = useArrayFilter(list, (item) => item % 2 === 0);
const doubled = useArrayMap(list, (item) => item * 2);
const found = useArrayFind(list, (item) => item > 1);
const sum = useArrayReduce(list, (total: number, item) => total + item, 0);
const some = useArraySome(list, (item) => item > 2);
const every = useArrayEvery(list, (item) => item > 0);
const unique = useArrayUnique(list);`,
    state: "{ even, doubled, found, sum, some, every, unique }",
    server: {
      even: [2, 2],
      doubled: [2, 4, 4, 6],
      found: 2,
      sum: 8,
      some: true,
      every: true,
      unique: [1, 2, 3],
    },
    client: {
      even: [2, 2],
      doubled: [2, 4, 4, 6],
      found: 2,
      sum: 8,
      some: true,
      every: true,
      unique: [1, 2, 3],
    },
  },
  "./use-async-queue": {
    kind: "component",
    setup: `import { useAsyncQueue } from "@/use-async-queue.ts";
const queue = useAsyncQueue([() => Promise.resolve(1), (previous: number) => Promise.resolve(previous + 1)], {
  immediate: false,
});`,
    state: "{ active: queue.activeIndex.value, finished: queue.isFinished.value }",
    server: { active: -1, finished: false },
    client: { active: -1, finished: false },
  },
  "./use-async-state": {
    kind: "component",
    setup: `import { useAsyncState } from "@/use-async-state.ts";
const async = useAsyncState(() => Promise.resolve("loaded"), "initial", { immediate: false });`,
    state:
      "{ state: async.state.value, ready: async.isReady.value, loading: async.isLoading.value }",
    server: { state: "initial", ready: false, loading: false },
    client: { state: "initial", ready: false, loading: false },
  },
  "./use-broadcast-channel": {
    kind: "component",
    setup: `import { useBroadcastChannel } from "@/use-broadcast-channel.ts";
// A client-only in-memory channel (happy-dom has no BroadcastChannel).
class FakeChannel extends EventTarget {
  readonly name: string;
  constructor(name: string) {
    super();
    this.name = name;
  }
  postMessage(_message: unknown): void {}
  close(): void {}
}
const host = typeof window === "undefined" ? undefined : { BroadcastChannel: FakeChannel };
const channel = useBroadcastChannel<string>("vize-conformance", { host: () => host });`,
    state:
      "{ supported: channel.supported.value, closed: channel.closed.value, data: channel.data.value ?? null }",
    server: { supported: false, closed: true, data: null },
    client: { supported: true, closed: false, data: null },
  },
  "./use-clipboard": {
    kind: "component",
    setup: `import { useClipboard } from "@/use-clipboard.ts";
const clipboard = useClipboard();`,
    state:
      "{ supported: clipboard.supported.value, text: clipboard.text.value, copied: clipboard.copied.value }",
    server: { supported: false, text: "", copied: false },
    client: { supported: true, text: "", copied: false },
  },
  "./use-confirm-dialog": {
    kind: "component",
    setup: `import { useConfirmDialog } from "@/use-confirm-dialog.ts";
const dialog = useConfirmDialog();`,
    state: "{ state: dialog.state.value, revealed: dialog.isRevealed.value }",
    server: { state: "idle", revealed: false },
    client: { state: "idle", revealed: false },
  },
};
