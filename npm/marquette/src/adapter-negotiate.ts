import type { ApplicationMarquette } from "./model.js";
import type {
  AdapterCapabilityManifest,
  AdapterCapabilityMismatch,
  AdapterCapabilityMismatchCode,
  AdapterCapabilityNegotiation,
  AdapterCapabilitySupport,
} from "./adapter-model.js";
import { validateAdapterCapabilityManifest } from "./adapter-validate.js";

/**
 * Negotiates application capability requirements against one adapter.
 *
 * Requirement identifiers are deduplicated and sorted. Unknown application
 * capability identifiers fail closed instead of being treated as adapter
 * omissions. An invalid manifest never produces a compatible result.
 */
export function negotiateAdapterCapabilities(
  marquette: ApplicationMarquette,
  requiredCapabilities: readonly string[],
  manifest: AdapterCapabilityManifest,
): AdapterCapabilityNegotiation {
  const diagnostics = validateAdapterCapabilityManifest(manifest);
  const support = new Map(
    (manifest.capabilities ?? []).map((capability) => [capability.id, capability] as const),
  );
  const mismatches = [...new Set(requiredCapabilities)]
    .sort()
    .flatMap((id) => mismatch(marquette, id, support.get(id), diagnostics.length === 0));

  return {
    adapter: manifest.adapter,
    compatible: diagnostics.length === 0 && mismatches.length === 0,
    diagnostics,
    mismatches,
  };
}

function mismatch(
  marquette: ApplicationMarquette,
  id: string,
  support: AdapterCapabilitySupport | undefined,
  manifestIsValid: boolean,
): AdapterCapabilityMismatch[] {
  const requirement = marquette.capabilities?.[id];
  if (requirement === undefined) return [problem("unknown-requirement", id)];
  if (!manifestIsValid) return [];

  const requiredVersion = requirement.version ?? 1;
  if (support === undefined) return [problem("missing-capability", id, requiredVersion)];
  if (requiredVersion < support.minVersion) {
    return [problem("version-below-minimum", id, requiredVersion, support)];
  }
  if (requiredVersion > support.maxVersion) {
    return [problem("version-above-maximum", id, requiredVersion, support)];
  }
  return [];
}

function problem(
  code: AdapterCapabilityMismatchCode,
  capability: string,
  requiredVersion?: number,
  support?: AdapterCapabilitySupport,
): AdapterCapabilityMismatch {
  return {
    code,
    capability,
    path: `capabilities.${capability}`,
    message: mismatchMessage(code),
    ...(requiredVersion === undefined ? {} : { requiredVersion }),
    ...(support === undefined
      ? {}
      : { minVersion: support.minVersion, maxVersion: support.maxVersion }),
  };
}

function mismatchMessage(code: AdapterCapabilityMismatchCode): string {
  switch (code) {
    case "unknown-requirement":
      return "application references an undeclared capability requirement";
    case "missing-capability":
      return "adapter does not support the required capability";
    case "version-below-minimum":
      return "application requires a capability version below the adapter minimum";
    case "version-above-maximum":
      return "application requires a capability version above the adapter maximum";
  }
}
