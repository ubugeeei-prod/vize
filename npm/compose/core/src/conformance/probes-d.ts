import type { ComposableProbeMap } from "./probe-types.ts";

/** Conformance probes for catalog entries in group D (test-only). */
export const probesGroupD: ComposableProbeMap = {
  "./media-query": {
    kind: "component",
    setup:
      'import { useMediaQuery } from "@/media-query.ts";\nconst wide = useMediaQuery("(min-width: 600px)");',
    state: "{ wide }",
    server: { wide: false },
    client: { wide: true },
  },
  "./use-previous": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { usePrevious } from "@/use-previous.ts";',
      "const count = ref(1);",
      "const previous = usePrevious(count, 0);",
    ].join("\n"),
    state: "{ count, previous }",
    server: { count: 1, previous: 0 },
    client: { count: 1, previous: 0 },
  },
  "./use-queue": {
    kind: "component",
    setup: [
      'import { useQueue } from "@/use-queue.ts";',
      "const queue = useQueue([1, 2], { capacity: 3 });",
      "queue.enqueue(3);",
      "const head = queue.dequeue();",
    ].join("\n"),
    state: "{ head, items: queue.items, size: queue.size, empty: queue.isEmpty }",
    server: { head: 1, items: [2, 3], size: 2, empty: false },
    client: { head: 1, items: [2, 3], size: 2, empty: false },
  },
  "./use-raf-fn": {
    kind: "component",
    setup: [
      'import { useRafFn } from "@/use-raf-fn.ts";',
      "const raf = useRafFn(() => undefined);",
    ].join("\n"),
    state: "{ active: raf.isActive }",
    server: { active: true },
    client: { active: true },
  },
  "./use-relative-time-format": {
    kind: "component",
    setup: [
      'import { useRelativeTimeFormat } from "@/use-relative-time-format.ts";',
      'const relative = useRelativeTimeFormat(-1, "day", { locale: "en-US" });',
    ].join("\n"),
    state: "{ text: relative.formatted }",
    server: { text: "1 day ago" },
    client: { text: "1 day ago" },
  },
  "./use-script-tag": {
    kind: "component",
    setup: [
      'import { useScriptTag } from "@/use-script-tag.ts";',
      'const script = useScriptTag("https://conformance.vize.test/probe.js", { immediate: false });',
    ].join("\n"),
    state: "{ status: script.status }",
    server: { status: "idle" },
    client: { status: "idle" },
  },
  "./use-segmenter": {
    kind: "component",
    setup: [
      'import { useSegmenter } from "@/use-segmenter.ts";',
      'const segmenter = useSegmenter("Hello brave world", { locale: "en-US", granularity: "word" });',
    ].join("\n"),
    state: "{ words: segmenter.wordCount, graphemes: segmenter.graphemeCount }",
    server: { words: 3, graphemes: 17 },
    client: { words: 3, graphemes: 17 },
  },
  "./use-selection": {
    kind: "component",
    setup: [
      'import { useSelection } from "@/use-selection.ts";',
      'const selection = useSelection({ items: ["a", "b", "c"], multiple: true, initial: ["a"] });',
      'selection.select("c");',
    ].join("\n"),
    state: "{ keys: selection.selectedKeys, count: selection.count }",
    server: { keys: ["a", "c"], count: 2 },
    client: { keys: ["a", "c"], count: 2 },
  },
  "./use-set": {
    kind: "component",
    setup: [
      'import { useSet } from "@/use-set.ts";',
      "const set = useSet([1, 2]);",
      "set.add(3);",
      "set.toggle(1);",
    ].join("\n"),
    state: "{ values: set.values, size: set.size }",
    server: { values: [2, 3], size: 2 },
    client: { values: [2, 3], size: 2 },
  },
  "./use-share": {
    kind: "component",
    setup: [
      'import { useShare } from "@/use-share.ts";',
      'const share = useShare({ title: "Vize" });',
    ].join("\n"),
    state: "{ supported: share.supported, sharing: share.sharing }",
    server: { supported: false, sharing: false },
  },
  "./use-sorted": {
    kind: "component",
    setup: [
      'import { useSorted } from "@/use-sorted.ts";',
      "const sorted = useSorted([3, 1, 2]);",
    ].join("\n"),
    state: "{ sorted }",
    server: { sorted: [1, 2, 3] },
    client: { sorted: [1, 2, 3] },
  },
  "./use-sorted-locale": {
    kind: "component",
    setup: [
      'import { useSortedLocale } from "@/use-sorted-locale.ts";',
      'const sorted = useSortedLocale(["b", "ä", "a"], { locale: "de" });',
    ].join("\n"),
    state: "{ sorted: sorted.sorted }",
    server: { sorted: ["a", "ä", "b"] },
    client: { sorted: ["a", "ä", "b"] },
  },
  "./use-speech-recognition": {
    kind: "component",
    setup: [
      'import { useSpeechRecognition } from "@/use-speech-recognition.ts";',
      'const recognition = useSpeechRecognition({ lang: "en-US" });',
    ].join("\n"),
    state: "{ supported: recognition.supported, listening: recognition.listening }",
    server: { supported: false, listening: false },
  },
  "./use-speech-synthesis": {
    kind: "component",
    setup: [
      'import { useSpeechSynthesis } from "@/use-speech-synthesis.ts";',
      'const speech = useSpeechSynthesis("Hello", { lang: "en-US" });',
    ].join("\n"),
    state: "{ supported: speech.supported, status: speech.status }",
    server: { supported: false, status: "idle" },
  },
  "./use-stack": {
    kind: "component",
    setup: [
      'import { useStack } from "@/use-stack.ts";',
      "const stack = useStack([1, 2]);",
      "stack.push(3);",
      "const top = stack.pop();",
    ].join("\n"),
    state: "{ top, items: stack.items, size: stack.size, peek: stack.peek() }",
    server: { top: 3, items: [1, 2], size: 2, peek: 2 },
    client: { top: 3, items: [1, 2], size: 2, peek: 2 },
  },
  "./use-stepper": {
    kind: "component",
    setup: [
      'import { useStepper } from "@/use-stepper.ts";',
      'const stepper = useStepper(["account", "profile", "done"], "profile");',
    ].join("\n"),
    state:
      "{ current: stepper.current, index: stepper.index, first: stepper.isFirst, last: stepper.isLast }",
    server: { current: "profile", index: 1, first: false, last: false },
    client: { current: "profile", index: 1, first: false, last: false },
  },
  "./use-storage": {
    kind: "component",
    setup:
      'import { useStorage } from "@/use-storage.ts";\nconst { state } = useStorage("probe", 1);',
    state: "{ state }",
    server: { state: 1 },
    client: { state: 1 },
  },
  "./use-style-tag": {
    kind: "component",
    setup: [
      'import { useStyleTag } from "@/use-style-tag.ts";',
      'const style = useStyleTag(".probe { color: red; }", { id: "vize-conformance-style" });',
    ].join("\n"),
    state: "{ id: style.id, loaded: style.loaded }",
    server: { id: "vize-conformance-style", loaded: false },
    client: { id: "vize-conformance-style", loaded: true },
  },
  "./use-supported": {
    kind: "component",
    setup: [
      'import { useSupported } from "@/use-supported.ts";',
      'const supported = useSupported(() => typeof window.matchMedia === "function");',
    ].join("\n"),
    state: "{ supported }",
    server: { supported: false },
    client: { supported: true },
  },
  "./use-throttled": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { useThrottled } from "@/use-throttled.ts";',
      "const source = ref(1);",
      "const throttled = useThrottled(source, 100);",
    ].join("\n"),
    state: "{ value: throttled.throttled, pending: throttled.pending }",
    server: { value: 1, pending: false },
    client: { value: 1, pending: false },
  },
  "./use-time-ago": {
    kind: "component",
    setup: [
      'import { useTimeAgo } from "@/use-time-ago.ts";',
      "const now = Date.UTC(2026, 0, 2);",
      'const timeAgo = useTimeAgo(Date.UTC(2026, 0, 1), { locale: "en-US", now: () => now });',
    ].join("\n"),
    state: "{ timeAgo }",
    server: { timeAgo: "yesterday" },
    client: { timeAgo: "yesterday" },
  },
  "./use-timeout": {
    kind: "component",
    setup: [
      'import { useTimeout } from "@/use-timeout.ts";',
      "const ready = useTimeout(60_000);",
    ].join("\n"),
    state: "{ ready }",
    server: { ready: false },
    client: { ready: false },
  },
  "./use-toggle": {
    kind: "component",
    setup: 'import { useToggle } from "@/use-toggle.ts";\nconst { state } = useToggle(true);',
    state: "{ state }",
    server: { state: true },
    client: { state: true },
  },
  "./use-url-hash": {
    kind: "component",
    setup: [
      'import { useUrlHash } from "@/use-url-hash.ts";',
      'const hash = useUrlHash({ ssrHash: "" });',
    ].join("\n"),
    state: "{ hash: hash.state, supported: hash.supported }",
    server: { hash: "", supported: false },
    client: { hash: "", supported: true },
  },
  "./use-url-search-params": {
    kind: "component",
    setup: [
      'import { searchParam, useUrlSearchParams } from "@/use-url-search-params.ts";',
      "const search = useUrlSearchParams(",
      '  { q: searchParam.string(""), page: searchParam.number(1) },',
      '  { ssrUrl: "https://conformance.vize.test/" },',
      ");",
    ].join("\n"),
    state: "{ params: search.params, supported: search.supported }",
    server: { params: { q: "", page: 1 }, supported: false },
    client: { params: { q: "", page: 1 }, supported: true },
  },
  "./use-user-media": {
    kind: "component",
    setup: [
      'import { useUserMedia } from "@/use-user-media.ts";',
      "const media = useUserMedia();",
    ].join("\n"),
    state: "{ supported: media.supported, status: media.status }",
    server: { supported: false, status: "idle" },
  },
  "./use-v-model": {
    kind: "component",
    setup: [
      'import { useVModel } from "@/use-v-model.ts";',
      "const props = defineProps<{ modelValue?: string }>();",
      'const emit = defineEmits<{ "update:modelValue": [value: string] }>();',
      'const model = useVModel(props, "modelValue", emit, { passive: true, defaultValue: "draft" });',
    ].join("\n"),
    state: "{ model }",
    server: { model: "draft" },
    client: { model: "draft" },
  },
  "./use-vibrate": {
    kind: "component",
    setup: [
      'import { useVibrate } from "@/use-vibrate.ts";',
      "const vibration = useVibrate({ pattern: [10] });",
    ].join("\n"),
    state: "{ supported: vibration.supported, vibrating: vibration.vibrating }",
    server: { supported: false, vibrating: false },
  },
  "./use-wake-lock": {
    kind: "component",
    setup: [
      'import { useWakeLock } from "@/use-wake-lock.ts";',
      "const wakeLock = useWakeLock();",
    ].join("\n"),
    state: "{ supported: wakeLock.supported, active: wakeLock.active }",
    server: { supported: false, active: false },
  },
  "./use-websocket": {
    kind: "component",
    setup: [
      'import { useWebSocket } from "@/use-websocket.ts";',
      'const socket = useWebSocket("wss://conformance.vize.test/socket", { immediate: false });',
    ].join("\n"),
    state: "{ status: socket.status, data: socket.data }",
    server: { status: "closed", data: null },
    client: { status: "closed", data: null },
  },
  "./use-worker-fn": {
    kind: "component",
    setup: [
      'import { useWorkerFn } from "@/use-worker-fn.ts";',
      "const worker = useWorkerFn((value: number) => value * 2);",
    ].join("\n"),
    state: "{ status: worker.status, supported: worker.supported }",
    server: { status: "idle", supported: false },
  },
  "./virtual-list": {
    kind: "component",
    setup: [
      'import { useVirtualList } from "@/virtual-list.ts";',
      "const items = Array.from({ length: 100 }, (_, index) => index);",
      "const virtual = useVirtualList(items, { itemSize: 20, initialItemCount: 5 });",
    ].join("\n"),
    state: "{ total: virtual.totalSize, rendered: virtual.list.value.map((entry) => entry.index) }",
    // Bind the ref and scroll handler only: happy-dom's CSSStyleDeclaration
    // exposes null values that crash Vue 3.6's dev-only Vapor style
    // hydration check, which is unrelated to the composable.
    markup:
      '<div :ref="virtual.containerProps.ref" @scroll="virtual.containerProps.onScroll"></div>',
    server: { total: 2000, rendered: [0, 1, 2, 3, 4] },
  },
  "./watch-debounced": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { watchDebounced } from "@/watch-debounced.ts";',
      "const source = ref(1);",
      "const seen = ref(0);",
      "watchDebounced(source, (value) => { seen.value = value; }, { debounce: 5 });",
    ].join("\n"),
    state: "{ seen }",
    server: { seen: 0 },
  },
  "./watch-ignorable": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { watchIgnorable } from "@/watch-ignorable.ts";',
      "const source = ref(1);",
      "const seen = ref(0);",
      'const { ignoreUpdates } = watchIgnorable(source, (value) => { seen.value = value; }, { flush: "sync" });',
      "ignoreUpdates(() => { source.value = 2; });",
      "source.value = 3;",
    ].join("\n"),
    state: "{ source, seen }",
    server: { source: 3, seen: 3 },
    client: { source: 3, seen: 3 },
  },
  "./watch-once": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { watchOnce } from "@/watch-once.ts";',
      "const source = ref(1);",
      "const calls = ref(0);",
      'watchOnce(source, () => { calls.value += 1; }, { flush: "sync" });',
      "source.value = 2;",
      "source.value = 3;",
    ].join("\n"),
    state: "{ calls }",
    server: { calls: 1 },
    client: { calls: 1 },
  },
  "./watch-pausable": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { watchPausable } from "@/watch-pausable.ts";',
      "const source = ref(1);",
      "const seen = ref(0);",
      'const watcher = watchPausable(source, (value) => { seen.value = value; }, { flush: "sync" });',
      "watcher.pause();",
      "source.value = 2;",
      "watcher.resume();",
      "source.value = 3;",
    ].join("\n"),
    state: "{ seen, active: watcher.isActive }",
    server: { seen: 3, active: true },
    client: { seen: 3, active: true },
  },
  "./watch-source": {
    kind: "exempt",
    reason: "Type-only module: it declares shared watch helper types and emits no runtime code.",
  },
  "./watch-throttled": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { watchThrottled } from "@/watch-throttled.ts";',
      "const source = ref(1);",
      "const seen = ref(0);",
      "watchThrottled(source, (value) => { seen.value = value; }, { throttle: 50, immediate: true });",
    ].join("\n"),
    state: "{ seen }",
  },
  "./whenever": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { whenever } from "@/whenever.ts";',
      "const flag = ref(false);",
      "const hits = ref(0);",
      'whenever(flag, () => { hits.value += 1; }, { flush: "sync" });',
      "flag.value = true;",
    ].join("\n"),
    state: "{ hits }",
    server: { hits: 1 },
    client: { hits: 1 },
  },
  "./window-size": {
    kind: "component",
    setup: [
      'import { useWindowSize } from "@/window-size.ts";',
      "const size = useWindowSize({ initialWidth: 0, initialHeight: 0 });",
    ].join("\n"),
    state: "{ width: size.width, height: size.height }",
    server: { width: 0, height: 0 },
    client: { width: 1024, height: 768 },
  },
};
