import assert from "node:assert/strict";
import { test } from "node:test";

import {
  availableCapability,
  isCapabilityAvailable,
  isCapabilityUnavailable,
  unavailableCapability,
} from "./capability.ts";
import type { CapabilityResult } from "./capability.ts";

void test("creates an available runtime capability without consulting globals", () => {
  const value = { vibrate: (milliseconds: number) => milliseconds > 0 };

  assert.deepEqual(availableCapability(value), {
    status: "available",
    available: true,
    value,
    source: "runtime",
  });
});

void test("preserves an adapter-specific source", () => {
  assert.deepEqual(availableCapability("clipboard", "desktop-host"), {
    status: "available",
    available: true,
    value: "clipboard",
    source: "desktop-host",
  });
});

void test("creates an unavailable capability with an explicit reason", () => {
  assert.deepEqual(unavailableCapability("insecure-context"), {
    status: "unavailable",
    available: false,
    reason: "insecure-context",
    details: undefined,
  });
});

void test("retains structured adapter diagnostics without interpreting them", () => {
  const details = { permission: "camera", canRequest: true } as const;

  assert.deepEqual(unavailableCapability("permission-denied", details), {
    status: "unavailable",
    available: false,
    reason: "permission-denied",
    details,
  });
});

void test("guards narrow both result branches", () => {
  const inspect = (result: CapabilityResult<number, "offline", { retry: boolean }>) => {
    if (isCapabilityAvailable(result)) return `value:${result.value}`;
    if (isCapabilityUnavailable(result)) {
      return `${result.reason}:${String(result.details.retry)}`;
    }
    return "unreachable";
  };

  assert.equal(inspect(availableCapability(42)), "value:42");
  assert.equal(inspect(unavailableCapability("offline", { retry: true })), "offline:true");
});
