import type { CapabilityTarget } from "./capability.ts";

/** Release maturity attached to every catalogued composable. */
export type ComposableStability = "experimental" | "stable" | "deprecated";

/** Functional area used to group composables in documentation and tooling. */
export type ComposableCategory =
  | "async"
  | "capability"
  | "events"
  | "i18n"
  | "lifecycle"
  | "media"
  | "networking"
  | "performance"
  | "state"
  | "storage"
  | "system"
  | "timing";

/** Server-rendering behavior of a composable when invoked without a browser. */
export type ComposableSsrBehavior = "safe" | "deterministic-fallback" | "unsupported";

/** Hydration responsibility associated with a composable's initial value. */
export type ComposableHydrationBehavior = "stable" | "caller-managed" | "not-applicable";

/** Party that can release resources retained by a composable. */
export type ComposableCleanupOwner = "none" | "caller" | "reactive-scope" | "returned-value";

/** Availability of one supported installation route. */
export type ComposableInstallationAvailability =
  | {
      /** The installation route can be used in the current release. */
      readonly status: "available";
      /** User-facing package or command specifier for the route. */
      readonly specifier: string;
    }
  | {
      /** The installation route has not been implemented yet. */
      readonly status: "unavailable";
      /** Stable explanation suitable for generated documentation. */
      readonly reason: string;
    };

/** Metadata for the aggregate package entry. */
export interface ComposableRootEntryMetadata {
  /** Export-map key for the aggregate entry. */
  readonly subpath: ".";
  /** Repository-relative source module used to build the entry. */
  readonly source: "src/index.ts";
  /** How the aggregate compressed-size budget is calculated. */
  readonly gzipBudget: "sum-of-reexported-entry-budgets";
  /** Feature entries re-exported by the aggregate entry. */
  readonly reexportedEntries: readonly `./${string}`[];
  /** Entries kept subpath-only to isolate comparatively heavy dependencies. */
  readonly isolatedEntries: readonly `./${string}`[];
}

/** Metadata for one independently importable package subpath. */
export interface ComposableEntryMetadata {
  /** Export-map key, including the leading `./`. */
  readonly subpath: `./${string}`;
  /** Repository-relative source module used to build the entry. */
  readonly source: `src/${string}.ts`;
  /** Maximum gzip bytes for the entry's complete emitted module closure. */
  readonly gzipBudgetBytes: number;
  /** Exact JavaScript exports emitted by this entry. */
  readonly runtimeExports: readonly string[];
  /** Runtime utility names implemented by this entry. */
  readonly utilities: readonly string[];
}

/** Operational metadata for one public runtime utility. */
export interface ComposableUtilityMetadata {
  /** Exact public export name. */
  readonly name: string;
  /** Package subpath that owns the utility's implementation. */
  readonly entry: `./${string}`;
  /** Functional documentation group. */
  readonly category: ComposableCategory;
  /** Release maturity of the current contract. */
  readonly stability: ComposableStability;
  /** Runtime families in which the utility is designed to operate. */
  readonly targets: readonly CapabilityTarget[];
  /** Behavior when called during server rendering. */
  readonly ssr: ComposableSsrBehavior;
  /** Initial-value responsibility at the hydration boundary. */
  readonly hydration: ComposableHydrationBehavior;
  /** Parties capable of releasing resources created by the utility. */
  readonly cleanupOwners: readonly ComposableCleanupOwner[];
  /** Host capability globals read at call time, excluding language intrinsics. */
  readonly runtimeGlobals: readonly string[];
  /** Other catalogued utilities called by this implementation. */
  readonly dependencies: readonly string[];
}

/** Machine-readable public contract for the composable package. */
export interface ComposableCatalog {
  /** Schema version for catalog consumers. */
  readonly schemaVersion: 1;
  /** Compatibility promise for the schema rather than individual utilities. */
  readonly catalogStability: "stable";
  /** Published package name. */
  readonly packageName: "@vizejs/composable";
  /** Canonical ordering of supported runtime families. */
  readonly targets: readonly CapabilityTarget[];
  /** Availability of package-manager installation. */
  readonly packageInstallation: ComposableInstallationAvailability;
  /** Availability of source-copy installation. */
  readonly sourceInstallation: ComposableInstallationAvailability;
  /** Aggregate package entry and its budget policy. */
  readonly rootEntry: ComposableRootEntryMetadata;
  /** Independently importable entries in export-map order. */
  readonly entries: readonly ComposableEntryMetadata[];
  /** Public runtime utilities in entry and source order. */
  readonly utilities: readonly ComposableUtilityMetadata[];
}

const allTargets = ["web", "server", "worker", "native", "desktop", "terminal"] as const;
const reactiveTargets = ["web", "server", "worker", "native", "desktop", "terminal"] as const;
const browserTargets = ["web", "desktop"] as const;

/**
 * Complete, serializable catalog of the package's public runtime surface.
 *
 * This value is the source of truth for export-map conformance and compressed
 * size gates. It intentionally contains data only: importing the module reads
 * no host globals, creates no subscriptions, and starts no work. Utilities
 * remain marked experimental while the package is pre-1.0. Source-copy
 * installation is reported as unavailable until a deterministic installer
 * with update provenance exists.
 */
export const COMPOSABLE_CATALOG = {
  schemaVersion: 1,
  catalogStability: "stable",
  packageName: "@vizejs/composable",
  targets: allTargets,
  packageInstallation: {
    status: "available",
    specifier: "@vizejs/composable",
  },
  sourceInstallation: {
    status: "unavailable",
    reason: "A provenance-preserving source-copy installer is not implemented.",
  },
  rootEntry: {
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
  },
  entries: [
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
  ],
  utilities: [
    {
      name: "anyAbortSignal",
      entry: "./abort-signal",
      category: "async",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["returned-value"],
      runtimeGlobals: ["AbortController", "AbortSignal"],
      dependencies: [],
    },
    {
      name: "timeoutAbortSignal",
      entry: "./abort-signal",
      category: "timing",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["returned-value"],
      runtimeGlobals: ["AbortController", "AbortSignal", "DOMException", "globalThis"],
      dependencies: ["anyAbortSignal"],
    },
    {
      name: "deadlineAbortSignal",
      entry: "./abort-signal",
      category: "timing",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["returned-value"],
      runtimeGlobals: [],
      dependencies: ["timeoutAbortSignal"],
    },
    {
      name: "useAsyncResource",
      entry: "./async-resource",
      category: "async",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "stable",
      cleanupOwners: ["caller", "reactive-scope"],
      runtimeGlobals: ["AbortController", "DOMException"],
      dependencies: ["tryOnScopeDispose"],
    },
    {
      name: "availableCapability",
      entry: "./capability",
      category: "capability",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "unavailableCapability",
      entry: "./capability",
      category: "capability",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "isCapabilityAvailable",
      entry: "./capability",
      category: "capability",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "isCapabilityUnavailable",
      entry: "./capability",
      category: "capability",
      stability: "experimental",
      targets: allTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "createDisposalScope",
      entry: "./disposal-scope",
      category: "lifecycle",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["caller", "reactive-scope"],
      runtimeGlobals: [],
      dependencies: ["tryOnScopeDispose"],
    },
    {
      name: "useEventListener",
      entry: "./event-listener",
      category: "events",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "stable",
      cleanupOwners: ["caller", "reactive-scope"],
      runtimeGlobals: [],
      dependencies: ["tryOnScopeDispose"],
    },
    {
      name: "useLocale",
      entry: "./locale",
      category: "i18n",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "deterministic-fallback",
      hydration: "caller-managed",
      cleanupOwners: ["none"],
      runtimeGlobals: ["Intl", "navigator"],
      dependencies: [],
    },
    {
      name: "useMediaQuery",
      entry: "./media-query",
      category: "media",
      stability: "experimental",
      targets: browserTargets,
      ssr: "deterministic-fallback",
      hydration: "caller-managed",
      cleanupOwners: ["reactive-scope"],
      runtimeGlobals: ["window"],
      dependencies: [],
    },
    {
      name: "useReducedMotion",
      entry: "./media-query",
      category: "media",
      stability: "experimental",
      targets: browserTargets,
      ssr: "deterministic-fallback",
      hydration: "caller-managed",
      cleanupOwners: ["reactive-scope"],
      runtimeGlobals: ["window"],
      dependencies: ["useMediaQuery"],
    },
    {
      name: "tryOnScopeDispose",
      entry: "./scope",
      category: "lifecycle",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "not-applicable",
      cleanupOwners: ["reactive-scope"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "useTemporalNow",
      entry: "./temporal",
      category: "timing",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "deterministic-fallback",
      hydration: "caller-managed",
      cleanupOwners: ["reactive-scope"],
      runtimeGlobals: ["globalThis", "window"],
      dependencies: [],
    },
    {
      name: "useTemporalZonedDateTime",
      entry: "./temporal",
      category: "timing",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "deterministic-fallback",
      hydration: "caller-managed",
      cleanupOwners: ["reactive-scope"],
      runtimeGlobals: ["globalThis", "window"],
      dependencies: ["useTemporalNow"],
    },
    {
      name: "useCounter",
      entry: "./use-counter",
      category: "state",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "stable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "useDebounced",
      entry: "./use-debounced",
      category: "timing",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "deterministic-fallback",
      hydration: "stable",
      cleanupOwners: ["caller", "reactive-scope"],
      runtimeGlobals: ["globalThis", "window"],
      dependencies: ["tryOnScopeDispose"],
    },
    {
      name: "useHistory",
      entry: "./use-history",
      category: "state",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "stable",
      cleanupOwners: ["reactive-scope"],
      runtimeGlobals: [],
      dependencies: ["tryOnScopeDispose"],
    },
    {
      name: "usePrevious",
      entry: "./use-previous",
      category: "state",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "stable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
    {
      name: "useThrottled",
      entry: "./use-throttled",
      category: "timing",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "deterministic-fallback",
      hydration: "stable",
      cleanupOwners: ["caller", "reactive-scope"],
      runtimeGlobals: ["globalThis", "window"],
      dependencies: ["tryOnScopeDispose"],
    },
    {
      name: "useToggle",
      entry: "./use-toggle",
      category: "state",
      stability: "experimental",
      targets: reactiveTargets,
      ssr: "safe",
      hydration: "stable",
      cleanupOwners: ["none"],
      runtimeGlobals: [],
      dependencies: [],
    },
  ],
} as const satisfies ComposableCatalog;
