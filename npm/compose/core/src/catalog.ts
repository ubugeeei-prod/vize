import { allTargets, entries, rootEntry } from "./catalog-entries.ts";
import type { ComposableCatalog } from "./catalog-schema.ts";
import { utilities } from "./catalog-utilities.ts";

export type {
  ComposableCatalog,
  ComposableCategory,
  ComposableCleanupOwner,
  ComposableEntryMetadata,
  ComposableHydrationBehavior,
  ComposableInstallationAvailability,
  ComposableRootEntryMetadata,
  ComposableSsrBehavior,
  ComposableStability,
  ComposableUtilityMetadata,
} from "./catalog-schema.ts";

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
  rootEntry,
  entries,
  utilities,
} as const satisfies ComposableCatalog;
