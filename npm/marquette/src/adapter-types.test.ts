import type {
  AdapterCapabilityDiagnosticCode,
  AdapterCapabilityManifest,
  AdapterCapabilityMismatch,
  AdapterCapabilityMismatchCode,
  CompatibilityChangeKind,
  NativeEngineCapabilityId,
} from "./adapter.js";

const manifest = {
  formatVersion: 1,
  adapter: "fixture.adapter",
  capabilities: [{ id: "auth.session", minVersion: 1, maxVersion: 3 }],
} as const satisfies AdapterCapabilityManifest;
const diagnostic: AdapterCapabilityDiagnosticCode = "duplicate-capability";
const mismatch: AdapterCapabilityMismatchCode = "version-above-maximum";
const mismatchDiagnostic = {
  code: "missing-capability",
  capability: "auth.session",
  path: "capabilities.auth.session",
  message: "adapter does not support the required capability",
  requiredVersion: 1,
} as const satisfies AdapterCapabilityMismatch;
const compatibility: CompatibilityChangeKind = "breaking";
const nativeCapability: NativeEngineCapabilityId = "native.accessibility";

// @ts-expect-error The serialized format is pinned to version one.
const future: AdapterCapabilityManifest = { formatVersion: 2, adapter: "future.adapter" };
const missingMaximum: AdapterCapabilityManifest = {
  adapter: "broken.adapter",
  // @ts-expect-error Supported ranges require both inclusive bounds.
  capabilities: [{ id: "broken", minVersion: 1 }],
};
// @ts-expect-error Native engine capability identifiers are a closed contract.
const unknownNativeCapability: NativeEngineCapabilityId = "native.unknown";

void manifest;
void diagnostic;
void mismatch;
void mismatchDiagnostic;
void compatibility;
void nativeCapability;
void future;
void missingMaximum;
void unknownNativeCapability;
