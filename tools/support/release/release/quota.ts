// Shared admission contract for cadence, preparation and publication.
// Internal policy reserves one below the observed 20-version rolling ceiling.
export { CRATE_QUOTA_BUDGET, CRATE_QUOTA_MARGIN_MS, CRATE_QUOTA_WINDOW_MS } from "./quota/types.ts";
export type {
  CrateQuotaObservation,
  CrateQuotaReport,
  RegistrySnapshot,
  ReleaseOwnership,
} from "./quota/types.ts";
export { inspectCrateQuota } from "./quota/inspect.ts";
export { readRollingCrateQuota } from "./quota/read.ts";
