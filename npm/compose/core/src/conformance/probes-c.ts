import type { ComposableProbeMap } from "./probe-types.ts";

/** Conformance probes for catalog entries in group C (test-only). */
export const probesGroupC: ComposableProbeMap = {
  "./use-cookie": {
    kind: "component",
    setup: [
      'import { useCookie } from "@/use-cookie.ts";',
      'const { state, supported } = useCookie("probe-theme", { default: "light" });',
    ].join("\n"),
    state: "{ state, supported }",
    server: { state: "light", supported: false },
    client: { state: "light", supported: true },
  },
  "./use-countdown": {
    kind: "component",
    setup: [
      'import { useCountdown } from "@/use-countdown.ts";',
      "const { remaining, isComplete } = useCountdown(3, { immediate: false });",
    ].join("\n"),
    state: "{ remaining, isComplete }",
    server: { remaining: 3, isComplete: false },
    client: { remaining: 3, isComplete: false },
  },
  "./use-counter": {
    kind: "component",
    setup: [
      'import { useCounter } from "@/use-counter.ts";',
      "const { count, atMin, atMax } = useCounter(2, { min: 0, max: 2 });",
    ].join("\n"),
    state: "{ count, atMin, atMax }",
    server: { count: 2, atMin: false, atMax: true },
    client: { count: 2, atMin: false, atMax: true },
  },
  "./use-css-var": {
    kind: "component",
    setup: [
      'import { useTemplateRef } from "vue";',
      'import { useCssVar } from "@/use-css-var.ts";',
      'const target = useTemplateRef<HTMLElement>("target");',
      'const { value, supported } = useCssVar("--probe-color", target, { initialValue: "red" });',
    ].join("\n"),
    markup: '<div ref="target"></div>',
    state: "{ value, supported }",
    server: { value: "red", supported: false },
    client: { value: "red", supported: true },
  },
  "./use-cycle-list": {
    kind: "component",
    setup: [
      'import { useCycleList } from "@/use-cycle-list.ts";',
      'const { state, index } = useCycleList(["a", "b", "c"] as const, { initialValue: "b" });',
    ].join("\n"),
    state: "{ state, index }",
    server: { state: "b", index: 1 },
    client: { state: "b", index: 1 },
  },
  "./use-date-format": {
    kind: "component",
    setup: [
      'import { useDateFormat } from "@/use-date-format.ts";',
      'const formatted = useDateFormat(Date.UTC(2026, 0, 2, 3, 4, 5), "YYYY-MM-DD HH:mm:ss");',
    ].join("\n"),
    state: "{ formatted }",
    server: { formatted: "2026-01-02 03:04:05" },
    client: { formatted: "2026-01-02 03:04:05" },
  },
  "./use-date-time-format": {
    kind: "component",
    setup: [
      'import { useDateTimeFormat } from "@/use-date-time-format.ts";',
      "const { formatted, locale } = useDateTimeFormat(Date.UTC(2026, 0, 2), {",
      '  locale: "en-US",',
      '  timeZone: "UTC",',
      '  dateStyle: "medium",',
      "});",
    ].join("\n"),
    state: "{ formatted, locale }",
    server: { formatted: "Jan 2, 2026", locale: "en-US" },
    client: { formatted: "Jan 2, 2026", locale: "en-US" },
  },
  "./use-debounced": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { useDebounced } from "@/use-debounced.ts";',
      'const source = ref("a");',
      "const { debounced, pending } = useDebounced(source, 50);",
    ].join("\n"),
    state: "{ debounced, pending }",
    server: { debounced: "a", pending: false },
    client: { debounced: "a", pending: false },
  },
  "./use-display-media": {
    kind: "component",
    setup: [
      'import { useDisplayMedia } from "@/use-display-media.ts";',
      "const { status, supported, stream } = useDisplayMedia();",
    ].join("\n"),
    state: "{ status, supported, active: Boolean(stream) }",
    server: { status: "idle", supported: false, active: false },
    // happy-dom implements no getDisplayMedia, so the client stays unsupported.
    client: { status: "idle", supported: false, active: false },
  },
  "./use-display-names": {
    kind: "component",
    setup: [
      'import { useDisplayNames } from "@/use-display-names.ts";',
      'const { name, locale } = useDisplayNames("JP", { type: "region", locale: "en-US" });',
    ].join("\n"),
    state: "{ name, locale }",
    server: { name: "Japan", locale: "en-US" },
    client: { name: "Japan", locale: "en-US" },
  },
  "./use-document-title": {
    kind: "component",
    setup: [
      'import { useDocumentTitle } from "@/use-document-title.ts";',
      'const { title, supported } = useDocumentTitle("Probe", { template: "%s | Vize" });',
    ].join("\n"),
    state: "{ title, supported }",
    server: { title: "Probe", supported: false },
    client: { title: "Probe", supported: true },
  },
  "./use-drop-zone": {
    kind: "component",
    setup: [
      'import { useTemplateRef } from "vue";',
      'import { useDropZone } from "@/use-drop-zone.ts";',
      'const zone = useTemplateRef<HTMLElement>("zone");',
      'const { isOverDropZone, accepted, files } = useDropZone(zone, { accept: ["image/*"] });',
    ].join("\n"),
    markup: '<div ref="zone"></div>',
    state: "{ isOverDropZone, accepted, files }",
    server: { isOverDropZone: false, accepted: false, files: null },
    client: { isOverDropZone: false, accepted: false, files: null },
  },
  "./use-event-source": {
    kind: "component",
    setup: [
      'import { useEventSource } from "@/use-event-source.ts";',
      "const { status, data, lastEventId } = useEventSource(null, { immediate: false });",
    ].join("\n"),
    state: "{ status, data, lastEventId }",
    server: { status: "closed", data: null, lastEventId: null },
    client: { status: "closed", data: null, lastEventId: null },
  },
  "./use-eye-dropper": {
    kind: "component",
    setup: [
      'import { useEyeDropper } from "@/use-eye-dropper.ts";',
      'const { sRGBHex, picking } = useEyeDropper({ initialValue: "#123456" });',
    ].join("\n"),
    state: "{ sRGBHex, picking }",
    server: { sRGBHex: "#123456", picking: false },
    client: { sRGBHex: "#123456", picking: false },
  },
  "./use-favicon": {
    kind: "component",
    setup: [
      'import { useFavicon } from "@/use-favicon.ts";',
      'const { icon, supported } = useFavicon("/probe.svg");',
    ].join("\n"),
    state: "{ icon, supported }",
    server: { icon: "/probe.svg", supported: false },
    client: { icon: "/probe.svg", supported: true },
  },
  "./use-fetch": {
    kind: "component",
    setup: [
      'import { useFetch } from "@/use-fetch.ts";',
      'const { status, data, pending } = useFetch("https://conformance.vize.test/api", { immediate: false });',
    ].join("\n"),
    state: "{ status, data, pending }",
    server: { status: "idle", data: null, pending: false },
    client: { status: "idle", data: null, pending: false },
  },
  "./use-field": {
    kind: "component",
    setup: [
      'import { useField } from "@/use-field.ts";',
      'const { value, valid, dirty } = useField("probe", { rules: (text) => (text ? undefined : "Required") });',
    ].join("\n"),
    state: "{ value, valid, dirty }",
    server: { value: "probe", valid: true, dirty: false },
    client: { value: "probe", valid: true, dirty: false },
  },
  "./use-file-dialog": {
    kind: "component",
    setup: [
      'import { useFileDialog } from "@/use-file-dialog.ts";',
      'const { files } = useFileDialog({ accept: "image/*" });',
    ].join("\n"),
    state: "{ files }",
    server: { files: null },
    client: { files: null },
  },
  "./use-file-system-access": {
    kind: "component",
    setup: [
      'import { useFileSystemAccess } from "@/use-file-system-access.ts";',
      "const { data, fileName, fileSize } = useFileSystemAccess();",
    ].join("\n"),
    state: "{ data, fileName, fileSize }",
    server: { data: null, fileName: "", fileSize: 0 },
    client: { data: null, fileName: "", fileSize: 0 },
  },
  "./use-form": {
    kind: "component",
    setup: [
      'import { useForm } from "@/use-form.ts";',
      "const form = useForm({",
      '  initialValues: { name: "Ada", tags: ["a", "b"] },',
      '  validators: { name: (name) => (name ? undefined : "Required") },',
      "});",
      'const name = form.field("name");',
      'const tags = form.fieldArray("tags");',
    ].join("\n"),
    state:
      "{ name: name.value.value, keys: tags.entries.value.map((entry) => entry.key), valid: form.valid.value, dirty: form.dirty.value }",
    server: { name: "Ada", keys: [0, 1], valid: true, dirty: false },
    client: { name: "Ada", keys: [0, 1], valid: true, dirty: false },
  },
  "./use-fullscreen": {
    kind: "component",
    setup: [
      'import { useFullscreen } from "@/use-fullscreen.ts";',
      "const { isFullscreen } = useFullscreen();",
    ].join("\n"),
    state: "{ isFullscreen }",
    server: { isFullscreen: false },
    client: { isFullscreen: false },
  },
  "./use-history": {
    kind: "component",
    setup: [
      'import { ref } from "vue";',
      'import { useHistory } from "@/use-history.ts";',
      "const source = ref(1);",
      "const { canUndo, canRedo, undoCount } = useHistory(source);",
    ].join("\n"),
    state: "{ source, canUndo, canRedo, undoCount }",
    server: { source: 1, canUndo: false, canRedo: false, undoCount: 0 },
    client: { source: 1, canUndo: false, canRedo: false, undoCount: 0 },
  },
  "./use-id-generator": {
    kind: "component",
    setup: [
      'import { useIdGenerator } from "@/use-id-generator.ts";',
      'const nextId = useIdGenerator({ prefix: "probe" });',
      'const ids = [nextId("input"), nextId("label")];',
    ].join("\n"),
    state: "{ ids }",
    server: { ids: ["probe-v-0-input-0", "probe-v-0-label-1"] },
    client: { ids: ["probe-v-0-input-0", "probe-v-0-label-1"] },
  },
  "./use-image": {
    kind: "component",
    setup: [
      'import { useImage } from "@/use-image.ts";',
      'const { status, ready } = useImage({ src: "/probe.png" }, { immediate: false });',
    ].join("\n"),
    state: "{ status, ready }",
    server: { status: "idle", ready: false },
    client: { status: "idle", ready: false },
  },
  "./use-indexed-db": {
    kind: "component",
    setup: [
      'import { useIndexedDB } from "@/use-indexed-db.ts";',
      'const { state, supported, ready } = useIndexedDB("probe", { count: 1 });',
    ].join("\n"),
    state: "{ state, supported, ready }",
    server: { state: { count: 1 }, supported: false, ready: false },
    // happy-dom implements no IndexedDB, so the client keeps the default.
    client: { state: { count: 1 }, supported: false, ready: false },
  },
  "./use-interval": {
    kind: "component",
    setup: [
      'import { useInterval } from "@/use-interval.ts";',
      "const { counter, isActive } = useInterval(60_000, { controls: true });",
    ].join("\n"),
    state: "{ counter, isActive }",
    server: { counter: 0, isActive: true },
    client: { counter: 0, isActive: true },
  },
  "./use-list-format": {
    kind: "component",
    setup: [
      'import { useListFormat } from "@/use-list-format.ts";',
      'const { formatted } = useListFormat(["a", "b", "c"], { locale: "en-US", type: "conjunction" });',
    ].join("\n"),
    state: "{ formatted }",
    server: { formatted: "a, b, and c" },
    client: { formatted: "a, b, and c" },
  },
  "./use-machine": {
    kind: "component",
    setup: [
      'import { useMachine } from "@/use-machine.ts";',
      "const machine = useMachine({",
      '  initial: "idle",',
      "  context: { count: 0 },",
      "  states: {",
      '    idle: { on: { START: { target: "running" } } },',
      '    running: { on: { STOP: { target: "idle" } } },',
      "  },",
      "});",
    ].join("\n"),
    state:
      "{ state: machine.state.value, context: machine.context.value, next: machine.nextEvents.value }",
    server: { state: "idle", context: { count: 0 }, next: ["START"] },
    client: { state: "idle", context: { count: 0 }, next: ["START"] },
  },
  "./use-map": {
    kind: "component",
    setup: [
      'import { useMap } from "@/use-map.ts";',
      'const { entries, size } = useMap([["a", 1], ["b", 2]]);',
    ].join("\n"),
    state: "{ entries, size }",
    server: {
      entries: [
        ["a", 1],
        ["b", 2],
      ],
      size: 2,
    },
    client: {
      entries: [
        ["a", 1],
        ["b", 2],
      ],
      size: 2,
    },
  },
  "./use-media-controls": {
    kind: "component",
    setup: [
      'import { useTemplateRef } from "vue";',
      'import { useMediaControls } from "@/use-media-controls.ts";',
      'const video = useTemplateRef<HTMLVideoElement>("video");',
      "const { playing, muted, volume, rate, duration, ended } = useMediaControls(video);",
    ].join("\n"),
    markup: '<video ref="video"></video>',
    state: "{ playing, muted, volume, rate, duration, ended }",
    server: { playing: false, muted: false, volume: 1, rate: 1, duration: 0, ended: false },
    client: { playing: false, muted: false, volume: 1, rate: 1, duration: 0, ended: false },
  },
  "./use-memoize": {
    kind: "component",
    setup: [
      'import { useMemoize } from "@/use-memoize.ts";',
      "const square = useMemoize((value: number) => value * value);",
      "const values = [square.load(3), square.load(3), square.load(4)];",
    ].join("\n"),
    state: "{ values }",
    server: { values: [9, 9, 16] },
    client: { values: [9, 9, 16] },
  },
  "./use-mounted": {
    kind: "component",
    setup: ['import { useMounted } from "@/use-mounted.ts";', "const mounted = useMounted();"].join(
      "\n",
    ),
    state: "{ mounted }",
    server: { mounted: false },
    client: { mounted: true },
  },
  "./use-notification": {
    kind: "component",
    setup: [
      'import { useWebNotification } from "@/use-notification.ts";',
      "const { permission, supported } = useWebNotification();",
    ].join("\n"),
    state: "{ permission, supported }",
    server: { permission: "unsupported", supported: false },
    // happy-dom implements no Notification constructor.
    client: { permission: "unsupported", supported: false },
  },
  "./use-now": {
    kind: "component",
    setup: [
      'import { useNow } from "@/use-now.ts";',
      "const now = useNow({ initial: Date.UTC(2026, 0, 1), immediate: false });",
    ].join("\n"),
    state: "{ now }",
    server: { now: "2026-01-01T00:00:00.000Z" },
    client: { now: "2026-01-01T00:00:00.000Z" },
  },
  "./use-number-format": {
    kind: "component",
    setup: [
      'import { useNumberFormat } from "@/use-number-format.ts";',
      'const { formatted } = useNumberFormat(1234.5, { locale: "en-US", style: "currency", currency: "USD" });',
    ].join("\n"),
    state: "{ formatted }",
    server: { formatted: "$1,234.50" },
    client: { formatted: "$1,234.50" },
  },
  "./use-object-url": {
    kind: "component",
    setup: [
      'import { useObjectUrl } from "@/use-object-url.ts";',
      "const url = useObjectUrl(null);",
    ].join("\n"),
    state: "{ url }",
    server: { url: null },
    client: { url: null },
  },
  "./use-offset-pagination": {
    kind: "component",
    setup: [
      'import { useOffsetPagination } from "@/use-offset-pagination.ts";',
      "const { currentPage, pageCount, offset, isFirstPage, isLastPage } = useOffsetPagination({",
      "  total: 95,",
      "  page: 2,",
      "  pageSize: 10,",
      "});",
    ].join("\n"),
    state: "{ currentPage, pageCount, offset, isFirstPage, isLastPage }",
    server: { currentPage: 2, pageCount: 10, offset: 10, isFirstPage: false, isLastPage: false },
    client: { currentPage: 2, pageCount: 10, offset: 10, isFirstPage: false, isLastPage: false },
  },
  "./use-permission": {
    kind: "component",
    setup: [
      'import { usePermission } from "@/use-permission.ts";',
      'const { state, supported } = usePermission("geolocation");',
    ].join("\n"),
    state: "{ state, supported }",
    server: { state: "unknown", supported: false },
    client: { state: "granted", supported: true },
  },
  "./use-plural-rules": {
    kind: "component",
    setup: [
      'import { usePluralRules } from "@/use-plural-rules.ts";',
      "const { category, message } = usePluralRules(",
      "  3,",
      '  { one: "# file", other: "# files" },',
      '  { locale: "en-US" },',
      ");",
    ].join("\n"),
    state: "{ category, message }",
    server: { category: "other", message: "3 files" },
    client: { category: "other", message: "3 files" },
  },
};
