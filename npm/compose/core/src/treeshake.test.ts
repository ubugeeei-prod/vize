import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import { build } from "vite-plus";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const sourceDirectory = fileURLToPath(new URL(".", import.meta.url));
const entries = {
  index: fileURLToPath(new URL("index.ts", import.meta.url)),
  temporal: fileURLToPath(new URL("temporal.ts", import.meta.url)),
} as const;

/**
 * Fingerprints that identify each source module inside an unminified
 * production bundle. Comments are stripped before matching, so identifiers
 * and string literals are reliable module markers: none of them appears in
 * another module or in the retained external `vue` import specifiers.
 */
const sentinels = {
  "abort-signal-any": ["anyAbortSignal"],
  "abort-signal-timeout": ["timeoutAbortSignal", "VIZE_COMPOSE_ABORT_TIMEOUT_INVALID_DELAY"],
  "abort-signal-deadline": ["deadlineAbortSignal", "VIZE_COMPOSE_ABORT_DEADLINE_INVALID"],
  scope: ["tryOnScopeDispose"],
  "capability-available": ['status: "available"'],
  "capability-unavailable": ['status: "unavailable"'],
  "capability-is-available": ["isCapabilityAvailable"],
  "capability-is-unavailable": ["isCapabilityUnavailable"],
  catalog: ["COMPOSABLE_CATALOG", "provenance-preserving source-copy installer"],
  "document-visibility": ["useDocumentVisibility", "readVisibilityState"],
  "disposal-scope": ["createDisposalScope", "VIZE_COMPOSE_DISPOSAL_FAILED"],
  "event-listener": ["useEventListener", "isListening"],
  "media-query": ["useMediaQuery"],
  "retry-async": ["retryAsync", "VIZE_COMPOSE_RETRY_INVALID_MAXIMUM_RETRIES"],
  "retry-delay": ["calculateRetryDelay", "VIZE_COMPOSE_RETRY_INVALID_ATTEMPT"],
  locale: ["useLocale", "getTextInfo"],
  "async-resource": ["useAsyncResource", "A newer execution started."],
  temporal: ["useTemporalNow", "VIZE_COMPOSE_TEMPORAL_INVALID_INTERVAL"],
  "use-previous": ["usePrevious"],
  "use-history": ["useHistory", "VIZE_COMPOSE_HISTORY_INVALID_CAPACITY"],
  "use-debounced": ["useDebounced", "VIZE_COMPOSE_DEBOUNCE_INVALID_WAIT"],
  "use-throttled": ["useThrottled", "VIZE_COMPOSE_THROTTLE_INVALID_WAIT"],
  "use-toggle": ["useToggle"],
  animate: ["function useAnimate("],
  breakpoints: ["function useBreakpoints("],
  "to-pixels": ["function toPixels("],
  "color-mode": ["function useColorMode(", "vize-color-mode"],
  dark: ["function useDark("],
  draggable: ["function useDraggable("],
  "element-by-point": ["function useElementByPoint("],
  "element-hover": ["function useElementHover("],
  "element-ref": ["function useElementRef("],
  "infinite-scroll": ["function useInfiniteScroll("],
  "on-click-outside": ["function onClickOutside("],
  "on-key-stroke": ["function onKeyStroke("],
  "on-key-down": ["function onKeyDown("],
  "on-key-up": ["function onKeyUp("],
  "on-long-press": ["function onLongPress("],
  "typing-keystroke": ["function isTypingKeystroke("],
  "on-start-typing": ["function onStartTyping("],
  "parent-element": ["function useParentElement("],
  "scroll-lock": ["function useScrollLock("],
  "textarea-autosize": ["function useTextareaAutosize("],
  transition: ["function useTransition("],
  "cubic-bezier": ["function cubicBezier("],
  "virtual-list": ["function useVirtualList("],
  "use-counter": ["useCounter", "VIZE_COMPOSE_COUNTER_INVALID_RANGE"],
  "active-element": ["function useActiveElement("],
  "element-bounding": ["function useElementBounding("],
  "element-target-node": ["function isElementNode("],
  "element-target-one": ["function resolveElement("],
  "element-target-many": ["function resolveElements("],
  focus: ["function useFocus("],
  "focus-within": ["function useFocusWithin("],
  "intersection-observer": ["function useIntersectionObserver("],
  "element-visibility": ["function useElementVisibility("],
  "key-name": ["function normalizeKeyName(", "arrowup"],
  "magic-keys": ["function useMagicKeys("],
  "key-pressed": ["function useKeyPressed("],
  mouse: ["function useMouse("],
  "mouse-in-element": ["function useMouseInElement("],
  "mutation-observer": ["function useMutationObserver("],
  pinch: ["function usePinch("],
  "pointer-kind": ["function toPointerKind("],
  pointer: ["function usePointer("],
  "pointer-lock": ["function usePointerLock(", "VIZE_COMPOSE_POINTER_LOCK_FAILED"],
  "resize-observer": ["function useResizeObserver("],
  "element-size": ["function useElementSize("],
  scroll: ["function useScroll("],
  "window-scroll": ["function useWindowScroll("],
  "swipe-direction": ["function swipeDirection("],
  swipe: ["function useSwipe("],
  "pointer-swipe": ["function usePointerSwipe("],
  "text-selection": ["function useTextSelection("],
  "window-size": ["function useWindowSize("],
  battery: ["function useBattery(", "chargingtimechange"],
  "device-motion": ["function useDeviceMotion("],
  "device-orientation": ["function useDeviceOrientation("],
  "motion-permission": ["function requestMotionPermission("],
  "device-pixel-ratio": ["function useDevicePixelRatio(", "dppx"],
  fps: ["function useFps("],
  geolocation: ["function useGeolocation("],
  idle: ["function useIdle("],
  network: ["function useNetwork("],
  online: ["function useOnline("],
  "page-leave": ["function usePageLeave("],
  "preferred-dark": ["function usePreferredDark("],
  "preferred-color-scheme": ["function usePreferredColorScheme("],
  "preferred-contrast": ["function usePreferredContrast(", "prefers-contrast"],
  "preferred-transparency": ["function usePreferredReducedTransparency("],
  "preferred-languages": ["function usePreferredLanguages(", "languagechange"],
  "screen-orientation": ["function useScreenOrientation("],
  "use-storage": ["useStorage", "vize:storage"],
  "use-storage-infer": ["inferStorageSerializerKind"],
  "use-indexed-db": ["useIndexedDB"],
  "use-indexed-db-keyval": ["createIndexedDBKeyval", "VIZE_COMPOSE_INDEXED_DB_UNAVAILABLE"],
  "use-cookie": ["useCookie"],
  "use-cookie-parse": ["parseCookieHeader"],
  "use-cookie-serialize": ["serializeCookie", "VIZE_COMPOSE_COOKIE_INVALID_NAME"],
  "use-cookie-key": ["vize:cookie-adapter"],
  "use-cookie-provide": ["provideCookieAdapter"],
  "use-url-search-params": ["useUrlSearchParams"],
  "use-url-search-params-parse": ["parseSearchParams"],
  "use-url-search-params-serialize": ["serializeSearchParams"],
  "use-url-hash": ["useUrlHash", "hashchange"],
  "use-broadcast-channel": ["useBroadcastChannel"],
  "use-fetch": ["useFetch", "VIZE_COMPOSE_FETCH_TIMEOUT"],
  "use-websocket": ["useWebSocket", "bufferWhileConnecting"],
  "use-event-source": ["useEventSource", "lastEventId"],
  "use-worker-fn": ["useWorkerFn"],
  "use-worker-fn-script": ["createWorkerFnScript", "__vizeWorkerFn"],
  "use-clipboard": ["useClipboard", "VIZE_COMPOSE_CLIPBOARD_INVALID_DURATION"],
  "use-css-var": ["useCssVar", "VIZE_COMPOSE_CSS_VAR_INVALID_NAME"],
  "use-display-media": ["useDisplayMedia", "getDisplayMedia"],
  "use-document-title": ["useDocumentTitle"],
  "use-drop-zone": ["useDropZone", "dragenter"],
  "use-drop-zone-accept": ["matchesAccept"],
  "use-eye-dropper": ["useEyeDropper"],
  "use-favicon": ["useFavicon"],
  "use-file-dialog": ["useFileDialog", "webkitdirectory"],
  "use-file-system-access": ["useFileSystemAccess", "showSaveFilePicker"],
  "use-fullscreen": ["useFullscreen", "webkitfullscreenchange"],
  "use-image": ["useImage"],
  "use-media-controls": ["useMediaControls", "enterpictureinpicture"],
  "use-notification": ["useWebNotification"],
  "use-object-url": ["useObjectUrl"],
  "use-permission": ["usePermission"],
  "use-script-tag": ["useScriptTag", "data-vize-script-status"],
  "use-share": ["useShare"],
  "use-speech-recognition": ["useSpeechRecognition", "webkitSpeechRecognition"],
  "use-speech-synthesis": ["useSpeechSynthesis", "VIZE_COMPOSE_SPEECH_SYNTHESIS_INVALID_OPTION"],
  "use-style-tag": ["useStyleTag", "vize-style"],
  "use-user-media": ["useMediaStream", "OverconstrainedError"],
  "use-vibrate": ["useVibrate", "VIZE_COMPOSE_VIBRATE_INVALID_PATTERN"],
  "use-wake-lock": ["useWakeLock"],
  until: ["VIZE_COMPOSE_UNTIL_TIMEOUT"],
  "use-countdown": ["useCountdown", "VIZE_COMPOSE_COUNTDOWN_INVALID_COUNT"],
  "use-date-format": ["formatDate", "VIZE_COMPOSE_DATE_FORMAT_INVALID_DATE"],
  "use-interval": ["useIntervalFn", "VIZE_COMPOSE_INTERVAL_INVALID_DELAY"],
  "use-now": ["createTimestamp"],
  "use-raf-fn": ["useRafFn", "VIZE_COMPOSE_RAF_INVALID_FPS_LIMIT"],
  "use-time-ago": ["formatTimeAgo", "VIZE_COMPOSE_TIME_AGO_INVALID_DATE"],
  "use-timeout": ["useTimeoutFn", "VIZE_COMPOSE_TIMEOUT_INVALID_DELAY"],
  "watch-debounced": ["watchDebounced", "VIZE_COMPOSE_WATCH_DEBOUNCED_INVALID_DELAY"],
  "watch-ignorable": ["watchIgnorable"],
  "watch-once": ["watchOnce"],
  "watch-pausable": ["watchPausable"],
  "watch-throttled": ["watchThrottled", "VIZE_COMPOSE_WATCH_THROTTLED_INVALID_DELAY"],
  whenever: ["whenever"],
  "computed-async": ["computedAsync"],
  "computed-with-control": ["computedWithControl"],
  "create-event-hook": ["createEventHook"],
  "create-global-state": ["createGlobalState", "VIZE_COMPOSE_GLOBAL_STATE_INACTIVE_SCOPE"],
  "create-injection-state": ["createInjectionState"],
  "create-shared-composable": [
    "createSharedComposable",
    "VIZE_COMPOSE_SHARED_COMPOSABLE_INACTIVE_SCOPE",
  ],
  reactify: ["reactify"],
  "ref-auto-reset": ["refAutoReset", "VIZE_COMPOSE_REF_AUTO_RESET_INVALID_DELAY"],
  "ref-debounced": ["refDebounced"],
  "ref-default": ["refDefault"],
  "sync-ref": ["syncRef"],
  "to-reactive": ["toReactive"],
  "use-id-generator": ["useIdGenerator"],
  "use-mounted": ["useMounted"],
  "use-supported": ["useSupported"],
  "use-v-model": ["useVModel"],
} as const;

type ModuleName = keyof typeof sentinels;

interface UtilityCase {
  readonly binding: string;
  readonly entry: keyof typeof entries;
  readonly module: ModuleName;
  /** Documented shared infrastructure allowed alongside the module itself. */
  readonly shared: readonly ModuleName[];
}

const utilities: readonly UtilityCase[] = [
  { binding: "anyAbortSignal", entry: "index", module: "abort-signal-any", shared: [] },
  {
    binding: "timeoutAbortSignal",
    entry: "index",
    module: "abort-signal-timeout",
    shared: ["abort-signal-any"],
  },
  {
    binding: "deadlineAbortSignal",
    entry: "index",
    module: "abort-signal-deadline",
    shared: ["abort-signal-timeout", "abort-signal-any"],
  },
  { binding: "tryOnScopeDispose", entry: "index", module: "scope", shared: [] },
  {
    binding: "availableCapability",
    entry: "index",
    module: "capability-available",
    shared: [],
  },
  {
    binding: "unavailableCapability",
    entry: "index",
    module: "capability-unavailable",
    shared: [],
  },
  {
    binding: "isCapabilityAvailable",
    entry: "index",
    module: "capability-is-available",
    shared: [],
  },
  {
    binding: "isCapabilityUnavailable",
    entry: "index",
    module: "capability-is-unavailable",
    shared: [],
  },
  {
    binding: "useDocumentVisibility",
    entry: "index",
    module: "document-visibility",
    shared: [],
  },
  {
    binding: "createDisposalScope",
    entry: "index",
    module: "disposal-scope",
    shared: ["scope"],
  },
  { binding: "useEventListener", entry: "index", module: "event-listener", shared: ["scope"] },
  { binding: "useMediaQuery", entry: "index", module: "media-query", shared: [] },
  { binding: "useReducedMotion", entry: "index", module: "media-query", shared: [] },
  {
    binding: "retryAsync",
    entry: "index",
    module: "retry-async",
    shared: ["abort-signal-timeout", "abort-signal-any", "retry-delay"],
  },
  { binding: "calculateRetryDelay", entry: "index", module: "retry-delay", shared: [] },
  { binding: "useLocale", entry: "index", module: "locale", shared: [] },
  { binding: "useAsyncResource", entry: "index", module: "async-resource", shared: ["scope"] },
  { binding: "useTemporalNow", entry: "temporal", module: "temporal", shared: [] },
  { binding: "useTemporalZonedDateTime", entry: "temporal", module: "temporal", shared: [] },
  { binding: "usePrevious", entry: "index", module: "use-previous", shared: [] },
  { binding: "useHistory", entry: "index", module: "use-history", shared: ["scope"] },
  { binding: "useDebounced", entry: "index", module: "use-debounced", shared: ["scope"] },
  { binding: "useThrottled", entry: "index", module: "use-throttled", shared: ["scope"] },
  { binding: "useToggle", entry: "index", module: "use-toggle", shared: [] },
  {
    binding: "useAnimate",
    entry: "index",
    module: "animate",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useBreakpoints",
    entry: "index",
    module: "breakpoints",
    shared: ["to-pixels", "media-query"],
  },
  { binding: "toPixels", entry: "index", module: "to-pixels", shared: [] },
  {
    binding: "useColorMode",
    entry: "index",
    module: "color-mode",
    shared: ["media-query", "element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useDark",
    entry: "index",
    module: "dark",
    shared: ["color-mode", "media-query", "element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useDraggable",
    entry: "index",
    module: "draggable",
    shared: ["element-target-one", "element-target-node", "pointer-kind", "scope"],
  },
  { binding: "useElementByPoint", entry: "index", module: "element-by-point", shared: ["scope"] },
  {
    binding: "useElementHover",
    entry: "index",
    module: "element-hover",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useElementRef",
    entry: "index",
    module: "element-ref",
    shared: ["element-target-one", "element-target-node"],
  },
  {
    binding: "useInfiniteScroll",
    entry: "index",
    module: "infinite-scroll",
    shared: ["scroll", "element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "onClickOutside",
    entry: "index",
    module: "on-click-outside",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "onKeyStroke",
    entry: "index",
    module: "on-key-stroke",
    shared: ["key-name", "scope"],
  },
  {
    binding: "onKeyDown",
    entry: "index",
    module: "on-key-down",
    shared: ["on-key-stroke", "key-name", "scope"],
  },
  {
    binding: "onKeyUp",
    entry: "index",
    module: "on-key-up",
    shared: ["on-key-stroke", "key-name", "scope"],
  },
  {
    binding: "onLongPress",
    entry: "index",
    module: "on-long-press",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  { binding: "isTypingKeystroke", entry: "index", module: "typing-keystroke", shared: [] },
  {
    binding: "onStartTyping",
    entry: "index",
    module: "on-start-typing",
    shared: ["typing-keystroke", "scope"],
  },
  {
    binding: "useParentElement",
    entry: "index",
    module: "parent-element",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useScrollLock",
    entry: "index",
    module: "scroll-lock",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useTextareaAutosize",
    entry: "index",
    module: "textarea-autosize",
    shared: [
      "resize-observer",
      "element-target-one",
      "element-target-many",
      "element-target-node",
      "scope",
    ],
  },
  {
    binding: "useTransition",
    entry: "index",
    module: "transition",
    shared: ["cubic-bezier", "scope"],
  },
  { binding: "cubicBezier", entry: "index", module: "cubic-bezier", shared: [] },
  {
    binding: "useVirtualList",
    entry: "index",
    module: "virtual-list",
    shared: [
      "element-ref",
      "resize-observer",
      "element-target-one",
      "element-target-many",
      "element-target-node",
      "scope",
    ],
  },
  { binding: "useCounter", entry: "index", module: "use-counter", shared: [] },
  { binding: "isElementNode", entry: "index", module: "element-target-node", shared: [] },
  {
    binding: "resolveElement",
    entry: "index",
    module: "element-target-one",
    shared: ["element-target-node"],
  },
  {
    binding: "resolveElements",
    entry: "index",
    module: "element-target-many",
    shared: ["element-target-node"],
  },
  { binding: "useActiveElement", entry: "index", module: "active-element", shared: ["scope"] },
  {
    binding: "useElementBounding",
    entry: "index",
    module: "element-bounding",
    shared: [
      "element-target-one",
      "element-target-many",
      "element-target-node",
      "event-listener",
      "resize-observer",
      "scope",
    ],
  },
  {
    binding: "useFocus",
    entry: "index",
    module: "focus",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useFocusWithin",
    entry: "index",
    module: "focus-within",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useIntersectionObserver",
    entry: "index",
    module: "intersection-observer",
    shared: ["element-target-one", "element-target-many", "element-target-node", "scope"],
  },
  {
    binding: "useElementVisibility",
    entry: "index",
    module: "element-visibility",
    shared: [
      "intersection-observer",
      "element-target-one",
      "element-target-many",
      "element-target-node",
      "scope",
    ],
  },
  { binding: "normalizeKeyName", entry: "index", module: "key-name", shared: [] },
  { binding: "useMagicKeys", entry: "index", module: "magic-keys", shared: ["key-name", "scope"] },
  {
    binding: "useKeyPressed",
    entry: "index",
    module: "key-pressed",
    shared: ["key-name", "scope"],
  },
  { binding: "useMouse", entry: "index", module: "mouse", shared: ["scope"] },
  {
    binding: "useMouseInElement",
    entry: "index",
    module: "mouse-in-element",
    shared: ["mouse", "element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useMutationObserver",
    entry: "index",
    module: "mutation-observer",
    shared: ["element-target-many", "element-target-node", "scope"],
  },
  {
    binding: "usePinch",
    entry: "index",
    module: "pinch",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  { binding: "toPointerKind", entry: "index", module: "pointer-kind", shared: [] },
  { binding: "usePointer", entry: "index", module: "pointer", shared: ["pointer-kind", "scope"] },
  {
    binding: "usePointerLock",
    entry: "index",
    module: "pointer-lock",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useResizeObserver",
    entry: "index",
    module: "resize-observer",
    shared: ["element-target-many", "element-target-node", "scope"],
  },
  {
    binding: "useElementSize",
    entry: "index",
    module: "element-size",
    shared: [
      "resize-observer",
      "element-target-one",
      "element-target-many",
      "element-target-node",
      "scope",
    ],
  },
  {
    binding: "useScroll",
    entry: "index",
    module: "scroll",
    shared: ["element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "useWindowScroll",
    entry: "index",
    module: "window-scroll",
    shared: ["scroll", "element-target-one", "element-target-node", "scope"],
  },
  { binding: "swipeDirection", entry: "index", module: "swipe-direction", shared: [] },
  {
    binding: "useSwipe",
    entry: "index",
    module: "swipe",
    shared: ["swipe-direction", "element-target-one", "element-target-node", "scope"],
  },
  {
    binding: "usePointerSwipe",
    entry: "index",
    module: "pointer-swipe",
    shared: [
      "swipe-direction",
      "pointer-kind",
      "element-target-one",
      "element-target-node",
      "scope",
    ],
  },
  { binding: "useTextSelection", entry: "index", module: "text-selection", shared: ["scope"] },
  { binding: "useWindowSize", entry: "index", module: "window-size", shared: ["scope"] },
  { binding: "useBattery", entry: "index", module: "battery", shared: ["scope"] },
  {
    binding: "useDeviceMotion",
    entry: "index",
    module: "device-motion",
    shared: ["motion-permission", "scope"],
  },
  {
    binding: "useDeviceOrientation",
    entry: "index",
    module: "device-orientation",
    shared: ["motion-permission", "scope"],
  },
  { binding: "requestMotionPermission", entry: "index", module: "motion-permission", shared: [] },
  {
    binding: "useDevicePixelRatio",
    entry: "index",
    module: "device-pixel-ratio",
    shared: ["scope"],
  },
  { binding: "useFps", entry: "index", module: "fps", shared: ["scope"] },
  {
    binding: "useGeolocation",
    entry: "index",
    module: "geolocation",
    shared: ["capability-available", "capability-unavailable", "scope"],
  },
  { binding: "useIdle", entry: "index", module: "idle", shared: ["scope"] },
  { binding: "useNetwork", entry: "index", module: "network", shared: ["scope"] },
  { binding: "useOnline", entry: "index", module: "online", shared: ["network", "scope"] },
  { binding: "usePageLeave", entry: "index", module: "page-leave", shared: ["scope"] },
  {
    binding: "usePreferredDark",
    entry: "index",
    module: "preferred-dark",
    shared: ["media-query"],
  },
  {
    binding: "usePreferredColorScheme",
    entry: "index",
    module: "preferred-color-scheme",
    shared: ["media-query"],
  },
  {
    binding: "usePreferredContrast",
    entry: "index",
    module: "preferred-contrast",
    shared: ["media-query"],
  },
  {
    binding: "usePreferredReducedTransparency",
    entry: "index",
    module: "preferred-transparency",
    shared: ["media-query"],
  },
  {
    binding: "usePreferredLanguages",
    entry: "index",
    module: "preferred-languages",
    shared: ["scope"],
  },
  {
    binding: "useScreenOrientation",
    entry: "index",
    module: "screen-orientation",
    shared: ["scope"],
  },
  {
    binding: "inferStorageSerializerKind",
    entry: "index",
    module: "use-storage-infer",
    shared: [],
  },
  {
    binding: "useStorage",
    entry: "index",
    module: "use-storage",
    shared: ["scope", "use-storage-infer"],
  },
  {
    binding: "useLocalStorage",
    entry: "index",
    module: "use-storage",
    shared: ["scope", "use-storage-infer"],
  },
  {
    binding: "useSessionStorage",
    entry: "index",
    module: "use-storage",
    shared: ["scope", "use-storage-infer"],
  },
  {
    binding: "createIndexedDBKeyval",
    entry: "index",
    module: "use-indexed-db-keyval",
    shared: [],
  },
  {
    binding: "useIndexedDB",
    entry: "index",
    module: "use-indexed-db",
    shared: ["scope", "use-indexed-db-keyval"],
  },
  {
    binding: "parseCookieHeader",
    entry: "index",
    module: "use-cookie-parse",
    shared: [],
  },
  {
    binding: "serializeCookie",
    entry: "index",
    module: "use-cookie-serialize",
    shared: [],
  },
  {
    binding: "provideCookieAdapter",
    entry: "index",
    module: "use-cookie-provide",
    shared: ["use-cookie-key"],
  },
  {
    binding: "useCookie",
    entry: "index",
    module: "use-cookie",
    shared: ["scope", "use-cookie-parse", "use-cookie-serialize", "use-cookie-key"],
  },
  {
    binding: "parseSearchParams",
    entry: "index",
    module: "use-url-search-params-parse",
    shared: [],
  },
  {
    binding: "serializeSearchParams",
    entry: "index",
    module: "use-url-search-params-serialize",
    shared: [],
  },
  {
    binding: "useUrlSearchParams",
    entry: "index",
    module: "use-url-search-params",
    shared: ["scope", "use-url-search-params-parse", "use-url-search-params-serialize"],
  },
  {
    binding: "useUrlHash",
    entry: "index",
    module: "use-url-hash",
    shared: ["scope"],
  },
  {
    binding: "useBroadcastChannel",
    entry: "index",
    module: "use-broadcast-channel",
    shared: ["scope"],
  },
  {
    binding: "useFetch",
    entry: "index",
    module: "use-fetch",
    shared: ["scope", "retry-async", "retry-delay", "abort-signal-timeout", "abort-signal-any"],
  },
  {
    binding: "useWebSocket",
    entry: "index",
    module: "use-websocket",
    shared: ["scope", "retry-delay"],
  },
  {
    binding: "useEventSource",
    entry: "index",
    module: "use-event-source",
    shared: ["scope", "retry-delay"],
  },
  {
    binding: "createWorkerFnScript",
    entry: "index",
    module: "use-worker-fn-script",
    shared: [],
  },
  {
    binding: "useWorkerFn",
    entry: "index",
    module: "use-worker-fn",
    shared: ["scope", "use-worker-fn-script"],
  },
  {
    binding: "useClipboard",
    entry: "index",
    module: "use-clipboard",
    shared: ["scope", "use-permission"],
  },
  {
    binding: "useCssVar",
    entry: "index",
    module: "use-css-var",
    shared: [],
  },
  {
    binding: "useDisplayMedia",
    entry: "index",
    module: "use-display-media",
    shared: ["use-user-media", "scope"],
  },
  {
    binding: "useDocumentTitle",
    entry: "index",
    module: "use-document-title",
    shared: ["scope"],
  },
  {
    binding: "matchesAccept",
    entry: "index",
    module: "use-drop-zone-accept",
    shared: [],
  },
  {
    binding: "useDropZone",
    entry: "index",
    module: "use-drop-zone",
    shared: ["scope", "use-drop-zone-accept"],
  },
  {
    binding: "useEyeDropper",
    entry: "index",
    module: "use-eye-dropper",
    shared: [],
  },
  {
    binding: "useFavicon",
    entry: "index",
    module: "use-favicon",
    shared: [],
  },
  {
    binding: "useFileDialog",
    entry: "index",
    module: "use-file-dialog",
    shared: ["scope"],
  },
  {
    binding: "useFileSystemAccess",
    entry: "index",
    module: "use-file-system-access",
    shared: [],
  },
  {
    binding: "useFullscreen",
    entry: "index",
    module: "use-fullscreen",
    shared: ["scope"],
  },
  {
    binding: "useImage",
    entry: "index",
    module: "use-image",
    shared: ["scope"],
  },
  {
    binding: "useMediaControls",
    entry: "index",
    module: "use-media-controls",
    shared: [],
  },
  {
    binding: "useWebNotification",
    entry: "index",
    module: "use-notification",
    shared: ["scope"],
  },
  {
    binding: "useObjectUrl",
    entry: "index",
    module: "use-object-url",
    shared: ["scope"],
  },
  {
    binding: "usePermission",
    entry: "index",
    module: "use-permission",
    shared: ["scope"],
  },
  {
    binding: "useScriptTag",
    entry: "index",
    module: "use-script-tag",
    shared: ["scope"],
  },
  {
    binding: "useShare",
    entry: "index",
    module: "use-share",
    shared: [],
  },
  {
    binding: "useSpeechRecognition",
    entry: "index",
    module: "use-speech-recognition",
    shared: [],
  },
  {
    binding: "useSpeechSynthesis",
    entry: "index",
    module: "use-speech-synthesis",
    shared: ["scope"],
  },
  {
    binding: "useStyleTag",
    entry: "index",
    module: "use-style-tag",
    shared: ["scope"],
  },
  {
    binding: "useMediaStream",
    entry: "index",
    module: "use-user-media",
    shared: ["scope"],
  },
  {
    binding: "useUserMedia",
    entry: "index",
    module: "use-user-media",
    shared: ["scope"],
  },
  {
    binding: "useVibrate",
    entry: "index",
    module: "use-vibrate",
    shared: ["scope"],
  },
  {
    binding: "useWakeLock",
    entry: "index",
    module: "use-wake-lock",
    shared: ["scope"],
  },
  { binding: "until", entry: "index", module: "until", shared: [] },
  {
    binding: "useCountdown",
    entry: "index",
    module: "use-countdown",
    shared: ["use-interval", "scope"],
  },
  { binding: "formatDate", entry: "index", module: "use-date-format", shared: [] },
  { binding: "useDateFormat", entry: "index", module: "use-date-format", shared: [] },
  { binding: "useIntervalFn", entry: "index", module: "use-interval", shared: ["scope"] },
  { binding: "useInterval", entry: "index", module: "use-interval", shared: ["scope"] },
  {
    binding: "useTimestamp",
    entry: "index",
    module: "use-now",
    shared: ["use-interval", "use-raf-fn", "scope"],
  },
  {
    binding: "useNow",
    entry: "index",
    module: "use-now",
    shared: ["use-interval", "use-raf-fn", "scope"],
  },
  { binding: "useRafFn", entry: "index", module: "use-raf-fn", shared: ["scope"] },
  { binding: "formatTimeAgo", entry: "index", module: "use-time-ago", shared: [] },
  {
    binding: "useTimeAgo",
    entry: "index",
    module: "use-time-ago",
    shared: ["use-now", "use-interval", "use-raf-fn", "scope"],
  },
  { binding: "useTimeoutFn", entry: "index", module: "use-timeout", shared: ["scope"] },
  { binding: "useTimeout", entry: "index", module: "use-timeout", shared: ["scope"] },
  { binding: "watchDebounced", entry: "index", module: "watch-debounced", shared: ["scope"] },
  { binding: "watchIgnorable", entry: "index", module: "watch-ignorable", shared: [] },
  { binding: "watchOnce", entry: "index", module: "watch-once", shared: [] },
  { binding: "watchPausable", entry: "index", module: "watch-pausable", shared: [] },
  { binding: "watchThrottled", entry: "index", module: "watch-throttled", shared: ["scope"] },
  { binding: "whenever", entry: "index", module: "whenever", shared: [] },
  { binding: "computedAsync", entry: "index", module: "computed-async", shared: [] },
  { binding: "computedWithControl", entry: "index", module: "computed-with-control", shared: [] },
  { binding: "createEventHook", entry: "index", module: "create-event-hook", shared: ["scope"] },
  { binding: "createGlobalState", entry: "index", module: "create-global-state", shared: [] },
  { binding: "createInjectionState", entry: "index", module: "create-injection-state", shared: [] },
  {
    binding: "createSharedComposable",
    entry: "index",
    module: "create-shared-composable",
    shared: ["scope"],
  },
  { binding: "reactify", entry: "index", module: "reactify", shared: [] },
  { binding: "refAutoReset", entry: "index", module: "ref-auto-reset", shared: ["scope"] },
  {
    binding: "refDebounced",
    entry: "index",
    module: "ref-debounced",
    shared: ["use-debounced", "scope"],
  },
  { binding: "refDefault", entry: "index", module: "ref-default", shared: [] },
  { binding: "syncRef", entry: "index", module: "sync-ref", shared: [] },
  { binding: "toReactive", entry: "index", module: "to-reactive", shared: [] },
  { binding: "useIdGenerator", entry: "index", module: "use-id-generator", shared: [] },
  { binding: "useMounted", entry: "index", module: "use-mounted", shared: [] },
  { binding: "useSupported", entry: "index", module: "use-supported", shared: ["use-mounted"] },
  { binding: "useVModel", entry: "index", module: "use-v-model", shared: [] },
  { binding: "useVModels", entry: "index", module: "use-v-model", shared: [] },
];

const VIRTUAL_ID = "virtual:compose-treeshake-entry";
const RESOLVED_ID = `\0${VIRTUAL_ID}`;

/**
 * Bundle a scratch entry in production mode, exactly like a consumer build.
 *
 * By default the package's own `"sideEffects": false` manifest hint is
 * neutralized: every `src` module is resolved with `moduleSideEffects: true`,
 * so statements survive on honest per-statement analysis alone. Without this,
 * the manifest lets the bundler drop a genuinely side-effectful module
 * wholesale and the emptiness assertions below would be vacuous.
 * `trustManifest` restores the consumer-visible semantics.
 */
async function bundleEntry(entryCode: string, trustManifest = false): Promise<string> {
  const result = await build({
    configFile: false,
    logLevel: "error",
    root: packageRoot,
    plugins: [
      {
        name: "compose-treeshake-entry",
        enforce: "pre",
        resolveId(id: string, importer: string | undefined) {
          if (id === VIRTUAL_ID) return RESOLVED_ID;
          if (trustManifest) return undefined;
          const importedFrom = importer === RESOLVED_ID ? packageRoot : dirname(importer ?? "/");
          const target = id.startsWith("./") ? resolve(importedFrom, id) : id;
          if (target.startsWith(sourceDirectory) && target.endsWith(".ts")) {
            return { id: target, moduleSideEffects: true };
          }
          return undefined;
        },
        load(id: string) {
          return id === RESOLVED_ID ? entryCode : undefined;
        },
      },
    ],
    build: {
      write: false,
      minify: false,
      target: "es2022",
      reportCompressedSize: false,
      rollupOptions: {
        input: VIRTUAL_ID,
        preserveEntrySignatures: "strict",
        // The peer and the runtime dependency stay external, matching how
        // `vp pack` builds the published entries.
        external: ["vue", "temporal-polyfill-lite"],
        // Both externals declare side-effect freedom in their own manifests;
        // unresolved externals would otherwise be conservatively retained.
        treeshake: { moduleSideEffects: "no-external" },
        output: { format: "es" },
      },
    },
  });
  assert.ok(!Array.isArray(result), "expected a single-environment build result");
  assert.ok("output" in result, "expected an in-memory rollup output");
  return result.output.map((item) => (item.type === "chunk" ? item.code : "")).join("\n");
}

/** Remove block comments and comment-only lines before sentinel matching. */
function stripComments(code: string): string {
  return code.replaceAll(/\/\*[^]*?\*\//g, "").replaceAll(/^[\t ]*\/\/.*$/gm, "");
}

void test("the package manifest declares side-effect freedom", () => {
  const manifest = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  ) as { sideEffects?: unknown };

  assert.equal(manifest.sideEffects, false);
});

void test("module-level code performs no work: a bare import bundles to nothing", async () => {
  for (const [name, entry] of Object.entries(entries)) {
    const code = stripComments(await bundleEntry(`import ${JSON.stringify(entry)};`));
    assert.equal(
      code.trim(),
      "",
      `entry "${name}" retained module-level side effects:\n${code.trim()}`,
    );
  }
});

void test("consumers honoring the manifest can drop the unused package wholesale", async () => {
  for (const [name, entry] of Object.entries(entries)) {
    const code = stripComments(await bundleEntry(`import ${JSON.stringify(entry)};`, true));
    assert.equal(code.trim(), "", `entry "${name}" must be fully removable when unused`);
  }
});

for (const { binding, entry, module, shared } of utilities) {
  void test(`bundling only ${binding} excludes every unrelated utility`, async () => {
    const code = stripComments(
      await bundleEntry(`export { ${binding} } from ${JSON.stringify(entries[entry])};`),
    );

    for (const sentinel of sentinels[module]) {
      assert.ok(code.includes(sentinel), `${binding} bundle lost its own marker "${sentinel}"`);
    }

    const allowed = new Set<ModuleName>([module, ...shared]);
    for (const other of Object.keys(sentinels) as ModuleName[]) {
      if (allowed.has(other)) continue;
      for (const sentinel of sentinels[other]) {
        assert.ok(
          !code.includes(sentinel),
          `${binding} bundle must not pull "${sentinel}" from ${other}`,
        );
      }
    }
  });
}

void test("bundling the catalog retains metadata without utility implementations", async () => {
  const code = stripComments(
    await bundleEntry(`export { COMPOSABLE_CATALOG } from ${JSON.stringify(entries.index)};`),
  );

  for (const sentinel of sentinels.catalog) {
    assert.ok(code.includes(sentinel), `catalog bundle lost its own marker "${sentinel}"`);
  }
  assert.doesNotMatch(code, /\bimport\s|\bfrom\s*["']/);
  assert.doesNotMatch(
    code,
    /\b(?:class|function)\s+(?:DisposalError|PointerLockError|anyAbortSignal|availableCapability|createDisposalScope|deadlineAbortSignal|isCapabilityAvailable|isCapabilityUnavailable|isElementNode|normalizeKeyName|requestMotionPermission|resolveElements?|swipeDirection|timeoutAbortSignal|toPointerKind|tryOnScopeDispose|unavailableCapability|on[A-Z]\w*|toPixels|cubicBezier|isTypingKeystroke|use[A-Z]\w*)\b/,
  );
});

void test("unused sibling exports of the same module are eliminated", async () => {
  const media = stripComments(
    await bundleEntry(`export { useMediaQuery } from ${JSON.stringify(entries.index)};`),
  );
  assert.ok(!media.includes("useReducedMotion"), "useReducedMotion should be dropped");
  assert.ok(!media.includes("prefers-reduced-motion"), "the motion query should be dropped");

  const temporal = stripComments(
    await bundleEntry(`export { useTemporalNow } from ${JSON.stringify(entries.temporal)};`),
  );
  assert.ok(
    !temporal.includes("useTemporalZonedDateTime"),
    "useTemporalZonedDateTime should be dropped",
  );
});

void test("importing an entry executes no module-level browser-global access", () => {
  for (const [name, entry] of Object.entries(entries)) {
    // The peer dependency is imported before the traps are installed, so the
    // probe measures this package (plus its bundled dependencies) only. The
    // throwing getters fail even a `typeof window` guard hoisted to module
    // scope, keeping capability detection lazy by construction. The probe
    // runs `src/*.ts` through the same type-stripping runtime as this suite.
    const probe = [
      'import "vue";',
      'for (const name of ["window", "document", "navigator"]) {',
      "  Object.defineProperty(globalThis, name, {",
      "    configurable: true,",
      "    get() {",
      "      throw new Error(`[import-side-effect] module-level access to globalThis.${name}`);",
      "    },",
      "  });",
      "}",
      `await import(${JSON.stringify(pathToFileURL(entry).href)});`,
    ].join("\n");

    const result = spawnSync(process.execPath, ["--input-type=module", "--eval", probe], {
      cwd: packageRoot,
      encoding: "utf8",
    });
    assert.equal(
      result.status,
      0,
      `entry "${name}" ran a module-level side effect:\n${result.stderr}`,
    );
  }
});
