import type {
  AdapterCapabilityDiagnosticCode,
  AdapterCapabilityManifest,
  AdapterCapabilityMismatchCode,
  CompatibilityChangeKind,
} from "./adapter.js";

const manifest = {
  formatVersion: 1,
  adapter: "fixture.adapter",
  capabilities: [{ id: "auth.session", minVersion: 1, maxVersion: 3 }],
} as const satisfies AdapterCapabilityManifest;
const diagnostic: AdapterCapabilityDiagnosticCode = "duplicate-capability";
const mismatch: AdapterCapabilityMismatchCode = "version-above-maximum";
const compatibility: CompatibilityChangeKind = "breaking";

// @ts-expect-error The serialized format is pinned to version one.
const future: AdapterCapabilityManifest = { formatVersion: 2, adapter: "future.adapter" };
const missingMaximum: AdapterCapabilityManifest = {
  adapter: "broken.adapter",
  // @ts-expect-error Supported ranges require both inclusive bounds.
  capabilities: [{ id: "broken", minVersion: 1 }],
};

void manifest;
void diagnostic;
void mismatch;
void compatibility;
void future;
void missingMaximum;
