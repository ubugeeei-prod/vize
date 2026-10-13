import assert from "node:assert/strict";
import { assertNativeTerminalLocks } from "./warm-type-backed-native-terminal-host.ts";
import { qualifyNativeTerminalHost } from "./warm-type-backed-native-owned-image-host.ts";
import { gitBodyDigest } from "./warm-type-backed-path-host.ts";
import { reviewedHostPaths } from "./warm-type-backed-reviewed-hosts.ts";
import { driverRoot } from "./warm-type-backed-source.ts";

type Binding = {
  production: string[];
  baseline: string;
  head: string;
  nativeTerminalHost?: ReturnType<typeof qualifyNativeTerminalHost>;
};

/** Reauthenticate the complete reviewed source at final original400 comparison. */
export function assertFinalNativeTerminalLocks(
  binding: Binding,
  before: Record<string, string>,
  after: Record<string, string>,
) {
  const qualified = qualifyNativeTerminalHost(
    binding.production,
    new Set(reviewedHostPaths),
    (side, file) =>
      gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );
  assert.deepEqual(qualified, binding.nativeTerminalHost ?? null);
  assertNativeTerminalLocks(before, after, qualified);
}
