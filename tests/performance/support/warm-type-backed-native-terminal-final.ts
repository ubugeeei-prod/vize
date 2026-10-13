import assert from "node:assert/strict";
import {
  qualifyOwnedCorsaHost,
  assertOwnedCorsaLocks,
} from "./warm-type-backed-owned-corsa-host.ts";
import { reviewedHostPaths } from "./warm-type-backed-reviewed-hosts.ts";
import { driverRoot } from "./warm-type-backed-source.ts";
import { assertNativeProxyFinalFixtureReceipt } from "./warm-type-backed-native-proxy-final-body.ts";

type Binding = {
  production: string[];
  baseline: string;
  head: string;
  nativeTerminalHost?: ReturnType<typeof qualifyOwnedCorsaHost>["nativeTerminalHost"];
  nativeProxyHost?: ReturnType<typeof qualifyOwnedCorsaHost>["nativeProxyHost"];
};

/** Reauthenticate the complete reviewed source at final original400 comparison. */
export function assertFinalNativeTerminalLocks(
  binding: Binding,
  before: Record<string, string>,
  after: Record<string, string>,
) {
  const allowed = new Set(reviewedHostPaths);
  const actual = qualifyOwnedCorsaHost(
    driverRoot,
    binding.baseline,
    binding.head,
    binding.production,
    allowed,
  );
  assertNativeProxyFinalFixtureReceipt(actual.nativeProxyHost, binding.nativeProxyHost);
  const qualified = actual.nativeTerminalHost;
  assert.deepEqual(qualified, binding.nativeTerminalHost ?? null);
  assertOwnedCorsaLocks(before, after, qualified);
}
