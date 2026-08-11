import type { ComposableEntryMetadata, ComposableRootEntryMetadata } from "./catalog-schema.ts";

/** Canonical runtime target ordering used by the catalog. */
export const allTargets = ["web", "server", "worker", "native", "desktop", "terminal"] as const;

/** Aggregate entry policy, including intentionally isolated subpaths. */
export const rootEntry = {
  subpath: ".",
  source: "src/index.ts",
  gzipBudget: "sum-of-reexported-entry-budgets",
  reexportedEntries: [
    "./abort-signal",
    "./async-resource",
    "./capability",
    "./catalog",
    "./disposal-scope",
    "./event-listener",
    "./locale",
    "./media-query",
    "./scope",
    "./timeout-scheduler",
    "./use-counter",
    "./use-debounced",
    "./use-history",
    "./use-previous",
    "./use-throttled",
    "./use-toggle",
  ],
  isolatedEntries: ["./temporal"],
} as const satisfies ComposableRootEntryMetadata;

/** Independently importable entries in package export-map order. */
export const entries = [
  {
    subpath: "./abort-signal",
    source: "src/abort-signal.ts",
    gzipBudgetBytes: 3 * 1024,
    runtimeExports: ["anyAbortSignal", "timeoutAbortSignal", "deadlineAbortSignal"],
    utilities: ["anyAbortSignal", "timeoutAbortSignal", "deadlineAbortSignal"],
  },
  {
    subpath: "./async-resource",
    source: "src/async-resource.ts",
    gzipBudgetBytes: 2 * 1024,
    runtimeExports: ["useAsyncResource"],
    utilities: ["useAsyncResource"],
  },
  {
    subpath: "./capability",
    source: "src/capability.ts",
    gzipBudgetBytes: 1024,
    runtimeExports: [
      "availableCapability",
      "unavailableCapability",
      "isCapabilityAvailable",
      "isCapabilityUnavailable",
    ],
    utilities: [
      "availableCapability",
      "unavailableCapability",
      "isCapabilityAvailable",
      "isCapabilityUnavailable",
    ],
  },
  {
    subpath: "./catalog",
    source: "src/catalog.ts",
    gzipBudgetBytes: 4 * 1024,
    runtimeExports: ["COMPOSABLE_CATALOG"],
    utilities: [],
  },
  {
    subpath: "./disposal-scope",
    source: "src/disposal-scope.ts",
    gzipBudgetBytes: 2 * 1024,
    runtimeExports: ["DISPOSAL_ERROR_CODE", "DisposalError", "createDisposalScope"],
    utilities: ["createDisposalScope"],
  },
  {
    subpath: "./event-listener",
    source: "src/event-listener.ts",
    gzipBudgetBytes: 2 * 1024,
    runtimeExports: ["useEventListener"],
    utilities: ["useEventListener"],
  },
  {
    subpath: "./locale",
    source: "src/locale.ts",
    gzipBudgetBytes: 2 * 1024,
    runtimeExports: ["useLocale"],
    utilities: ["useLocale"],
  },
  {
    subpath: "./media-query",
    source: "src/media-query.ts",
    gzipBudgetBytes: 1024,
    runtimeExports: ["useMediaQuery", "useReducedMotion"],
    utilities: ["useMediaQuery", "useReducedMotion"],
  },
  {
    subpath: "./scope",
    source: "src/scope.ts",
    gzipBudgetBytes: 0.5 * 1024,
    runtimeExports: ["tryOnScopeDispose"],
    utilities: ["tryOnScopeDispose"],
  },
  {
    subpath: "./temporal",
    source: "src/temporal.ts",
    gzipBudgetBytes: 2 * 1024,
    runtimeExports: ["Temporal", "TemporalIntl", "useTemporalNow", "useTemporalZonedDateTime"],
    utilities: ["useTemporalNow", "useTemporalZonedDateTime"],
  },
  {
    subpath: "./timeout-scheduler",
    source: "src/timeout-scheduler.ts",
    gzipBudgetBytes: 0.25 * 1024,
    runtimeExports: [],
    utilities: [],
  },
  {
    subpath: "./use-counter",
    source: "src/use-counter.ts",
    gzipBudgetBytes: 1.5 * 1024,
    runtimeExports: ["useCounter"],
    utilities: ["useCounter"],
  },
  {
    subpath: "./use-debounced",
    source: "src/use-debounced.ts",
    gzipBudgetBytes: 2 * 1024,
    runtimeExports: ["useDebounced"],
    utilities: ["useDebounced"],
  },
  {
    subpath: "./use-history",
    source: "src/use-history.ts",
    gzipBudgetBytes: 2.5 * 1024,
    runtimeExports: ["useHistory"],
    utilities: ["useHistory"],
  },
  {
    subpath: "./use-previous",
    source: "src/use-previous.ts",
    gzipBudgetBytes: 1024,
    runtimeExports: ["usePrevious"],
    utilities: ["usePrevious"],
  },
  {
    subpath: "./use-throttled",
    source: "src/use-throttled.ts",
    gzipBudgetBytes: 2.5 * 1024,
    runtimeExports: ["useThrottled"],
    utilities: ["useThrottled"],
  },
  {
    subpath: "./use-toggle",
    source: "src/use-toggle.ts",
    gzipBudgetBytes: 0.5 * 1024,
    runtimeExports: ["useToggle"],
    utilities: ["useToggle"],
  },
] as const satisfies readonly ComposableEntryMetadata[];
