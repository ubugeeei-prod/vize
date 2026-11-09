import {
  NATIVE_BUILD_PROFILE_FORMAT,
  NATIVE_BUILD_PROFILE_FORMAT_VERSION,
  type NativeBuildProfile,
  type NativeBuildProfileDiagnostic,
  type NativeBuildProfileDiagnosticCode,
} from "./native-build-model.js";

const IDENTIFIER = /^[a-z0-9][a-z0-9._-]*$/;
const RUNTIME_VERSION = /^[0-9A-Za-z][0-9A-Za-z._+-]*$/;
const CONFIGURATIONS = new Set(["development", "preview", "production"]);
const PLATFORMS = new Set(["android", "ios"]);
const DESTINATIONS = new Set(["simulator", "device", "store"]);
const SIGNING_MODES = new Set(["none", "automatic", "manual"]);
const DIAGNOSTIC_ORDER: Record<NativeBuildProfileDiagnosticCode, number> = {
  "invalid-format": 0,
  "invalid-format-version": 1,
  "invalid-profile-id": 2,
  "invalid-environment-id": 3,
  "missing-target": 4,
  "duplicate-target": 5,
  "store-requires-production": 6,
  "store-requires-signing": 7,
  "invalid-update-channel": 8,
  "invalid-runtime-version": 9,
  "invalid-capability-id": 10,
  "duplicate-capability": 11,
};

/** Validates a native build profile without mutating it. */
export function validateNativeBuildProfile(
  profile: NativeBuildProfile,
): NativeBuildProfileDiagnostic[] {
  const diagnostics: NativeBuildProfileDiagnostic[] = [];
  if (profile.format !== NATIVE_BUILD_PROFILE_FORMAT) {
    diagnostics.push(
      diagnostic("invalid-format", "format", "unsupported native build profile format"),
    );
  }
  if (profile.formatVersion !== NATIVE_BUILD_PROFILE_FORMAT_VERSION) {
    diagnostics.push(
      diagnostic(
        "invalid-format-version",
        "formatVersion",
        "unsupported native build profile format version",
      ),
    );
  }
  if (!IDENTIFIER.test(profile.profile)) {
    diagnostics.push(
      diagnostic(
        "invalid-profile-id",
        "profile",
        "profile must be a lowercase portable identifier",
      ),
    );
  }
  if (!IDENTIFIER.test(profile.environment)) {
    diagnostics.push(
      diagnostic(
        "invalid-environment-id",
        "environment",
        "environment must be a lowercase portable identifier",
      ),
    );
  }
  if (profile.targets.length === 0) {
    diagnostics.push(
      diagnostic("missing-target", "targets", "native build profile must declare a target"),
    );
  }

  const seenTargets = new Set<string>();
  for (const [index, target] of profile.targets.entries()) {
    const path = `targets.${index}`;
    const key = `${target.platform}:${target.destination}`;
    if (seenTargets.has(key)) {
      diagnostics.push(
        diagnostic(
          "duplicate-target",
          path,
          "platform and destination must be unique within the native build profile",
        ),
      );
    }
    seenTargets.add(key);
    if (target.destination === "store" && profile.configuration !== "production") {
      diagnostics.push(
        diagnostic(
          "store-requires-production",
          path,
          "store targets require the production configuration",
        ),
      );
    }
    if (target.destination === "store" && (target.signing ?? "none") === "none") {
      diagnostics.push(
        diagnostic(
          "store-requires-signing",
          `${path}.signing`,
          "store targets require automatic or manual signing",
        ),
      );
    }
  }

  if (profile.updates !== undefined) {
    if (!IDENTIFIER.test(profile.updates.channel)) {
      diagnostics.push(
        diagnostic(
          "invalid-update-channel",
          "updates.channel",
          "update channel must be a lowercase portable identifier",
        ),
      );
    }
    if (!RUNTIME_VERSION.test(profile.updates.runtimeVersion)) {
      diagnostics.push(
        diagnostic(
          "invalid-runtime-version",
          "updates.runtimeVersion",
          "runtime version must be a portable non-empty version token",
        ),
      );
    }
  }

  const seenCapabilities = new Set<string>();
  for (const [index, capability] of (profile.capabilities ?? []).entries()) {
    const path = `capabilities.${index}`;
    if (!IDENTIFIER.test(capability)) {
      diagnostics.push(
        diagnostic(
          "invalid-capability-id",
          path,
          "capability id must be a lowercase portable identifier",
        ),
      );
    }
    if (seenCapabilities.has(capability)) {
      diagnostics.push(
        diagnostic(
          "duplicate-capability",
          path,
          "capability id must be unique within the native build profile",
        ),
      );
    }
    seenCapabilities.add(capability);
  }

  return diagnostics.sort(
    (left, right) =>
      compareText(left.path, right.path) ||
      DIAGNOSTIC_ORDER[left.code] - DIAGNOSTIC_ORDER[right.code] ||
      compareText(left.message, right.message),
  );
}

/**
 * Parses an untrusted native build profile with strict structural checks.
 *
 * Unknown fields and mismatched primitive or enum values throw before a local
 * or hosted executor can inspect semantic diagnostics.
 */
export function parseNativeBuildProfile(value: unknown): NativeBuildProfile {
  assertRecord(value, "profile");
  assertKnownFields(
    value,
    [
      "format",
      "formatVersion",
      "profile",
      "configuration",
      "environment",
      "targets",
      "capabilities",
      "updates",
    ],
    "profile",
  );
  assertString(value.format, "format");
  if (
    typeof value.formatVersion !== "number" ||
    !Number.isSafeInteger(value.formatVersion) ||
    value.formatVersion < 0
  ) {
    throw new TypeError("formatVersion must be a safe integer");
  }
  assertString(value.profile, "profile");
  assertEnum(value.configuration, CONFIGURATIONS, "configuration");
  assertString(value.environment, "environment");
  if (!Array.isArray(value.targets)) throw new TypeError("targets must be an array");
  for (const [index, target] of value.targets.entries()) {
    const path = `targets.${index}`;
    assertRecord(target, path);
    assertKnownFields(target, ["platform", "destination", "signing"], path);
    assertEnum(target.platform, PLATFORMS, `${path}.platform`);
    assertEnum(target.destination, DESTINATIONS, `${path}.destination`);
    if (target.signing !== undefined) {
      assertEnum(target.signing, SIGNING_MODES, `${path}.signing`);
    }
  }
  if (value.capabilities !== undefined) {
    if (!Array.isArray(value.capabilities)) {
      throw new TypeError("capabilities must be an array");
    }
    for (const [index, capability] of value.capabilities.entries()) {
      assertString(capability, `capabilities.${index}`);
    }
  }
  if (value.updates !== undefined) {
    assertRecord(value.updates, "updates");
    assertKnownFields(value.updates, ["channel", "runtimeVersion"], "updates");
    assertString(value.updates.channel, "updates.channel");
    assertString(value.updates.runtimeVersion, "updates.runtimeVersion");
  }
  return value as unknown as NativeBuildProfile;
}

function diagnostic(
  code: NativeBuildProfileDiagnosticCode,
  path: string,
  message: string,
): NativeBuildProfileDiagnostic {
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
  const unknown = Object.keys(value).find((field) => !known.has(field));
  if (unknown !== undefined) throw new TypeError(`${path} has unknown field ${unknown}`);
}

function assertString(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string") throw new TypeError(`${path} must be a string`);
}

function assertEnum(value: unknown, expected: ReadonlySet<string>, path: string): void {
  if (typeof value !== "string" || !expected.has(value)) {
    throw new TypeError(`${path} must be a supported string value`);
  }
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
