import {
  ADAPTER_CAPABILITY_FORMAT_VERSION,
  type AdapterCapabilityDiagnostic,
  type AdapterCapabilityDiagnosticCode,
  type AdapterCapabilityManifest,
} from "./adapter-model.js";

const IDENTIFIER = /^[a-z0-9][a-z0-9._-]*$/;
const DIAGNOSTIC_ORDER: Record<AdapterCapabilityDiagnosticCode, number> = {
  "invalid-format-version": 0,
  "invalid-adapter-id": 1,
  "invalid-capability-id": 2,
  "invalid-version": 3,
  "invalid-version-range": 4,
  "duplicate-capability": 5,
};

/** Validates an adapter capability manifest without mutating it. */
export function validateAdapterCapabilityManifest(
  manifest: AdapterCapabilityManifest,
): AdapterCapabilityDiagnostic[] {
  const diagnostics: AdapterCapabilityDiagnostic[] = [];
  if ((manifest.formatVersion ?? 1) !== ADAPTER_CAPABILITY_FORMAT_VERSION) {
    diagnostics.push(
      diagnostic(
        "invalid-format-version",
        "formatVersion",
        "unsupported adapter capability manifest format version",
      ),
    );
  }
  if (!IDENTIFIER.test(manifest.adapter)) {
    diagnostics.push(
      diagnostic(
        "invalid-adapter-id",
        "adapter",
        "adapter must be a lowercase portable identifier",
      ),
    );
  }

  const seen = new Set<string>();
  for (const [index, capability] of (manifest.capabilities ?? []).entries()) {
    const path = `capabilities.${index}`;
    if (!IDENTIFIER.test(capability.id)) {
      diagnostics.push(
        diagnostic(
          "invalid-capability-id",
          `${path}.id`,
          "capability id must be a lowercase portable identifier",
        ),
      );
    }
    if (seen.has(capability.id)) {
      diagnostics.push(
        diagnostic(
          "duplicate-capability",
          `${path}.id`,
          "capability id must be unique within the adapter manifest",
        ),
      );
    }
    seen.add(capability.id);
    if (capability.minVersion <= 0) {
      diagnostics.push(
        diagnostic(
          "invalid-version",
          `${path}.minVersion`,
          "minimum supported version must be greater than zero",
        ),
      );
    }
    if (capability.maxVersion <= 0) {
      diagnostics.push(
        diagnostic(
          "invalid-version",
          `${path}.maxVersion`,
          "maximum supported version must be greater than zero",
        ),
      );
    }
    if (capability.minVersion > capability.maxVersion) {
      diagnostics.push(
        diagnostic(
          "invalid-version-range",
          path,
          "minimum supported version must not exceed maximum supported version",
        ),
      );
    }
  }

  return diagnostics.sort(
    (left, right) =>
      compareText(left.path, right.path) ||
      DIAGNOSTIC_ORDER[left.code] - DIAGNOSTIC_ORDER[right.code] ||
      compareText(left.message, right.message),
  );
}

/**
 * Parses an untrusted manifest with strict field and primitive checks.
 *
 * Unknown fields, missing required fields, and mismatched primitive types
 * throw before negotiation. Semantic range errors remain available from
 * {@link validateAdapterCapabilityManifest}.
 */
export function parseAdapterCapabilityManifest(value: unknown): AdapterCapabilityManifest {
  assertRecord(value, "manifest");
  assertKnownFields(value, ["formatVersion", "adapter", "capabilities"], "manifest");
  if (value.formatVersion !== undefined && value.formatVersion !== 1) {
    throw new TypeError("formatVersion must equal 1");
  }
  if (typeof value.adapter !== "string") {
    throw new TypeError("adapter must be a string");
  }
  if (value.capabilities !== undefined) {
    if (!Array.isArray(value.capabilities)) {
      throw new TypeError("capabilities must be an array");
    }
    for (const [index, capability] of value.capabilities.entries()) {
      const path = `capabilities.${index}`;
      assertRecord(capability, path);
      assertKnownFields(capability, ["id", "minVersion", "maxVersion"], path);
      if (typeof capability.id !== "string") throw new TypeError(`${path}.id must be a string`);
      if (
        typeof capability.minVersion !== "number" ||
        !Number.isSafeInteger(capability.minVersion) ||
        capability.minVersion < 0
      ) {
        throw new TypeError(`${path}.minVersion must be a safe integer`);
      }
      if (
        typeof capability.maxVersion !== "number" ||
        !Number.isSafeInteger(capability.maxVersion) ||
        capability.maxVersion < 0
      ) {
        throw new TypeError(`${path}.maxVersion must be a safe integer`);
      }
    }
  }
  return value as unknown as AdapterCapabilityManifest;
}

function diagnostic(
  code: AdapterCapabilityDiagnosticCode,
  path: string,
  message: string,
): AdapterCapabilityDiagnostic {
  return { code, path, message };
}

function assertRecord(value: unknown, path: string): asserts value is Record<string, unknown> {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${path} must be an object`);
  }
}

function assertKnownFields(
  value: Record<string, unknown>,
  expected: readonly string[],
  path: string,
): void {
  const known = new Set(expected);
  const unknown = Object.keys(value)
    .filter((field) => !known.has(field))
    .sort()[0];
  if (unknown !== undefined) throw new TypeError(`${path} has unknown field ${unknown}`);
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
