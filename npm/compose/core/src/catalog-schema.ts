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
