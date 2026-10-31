import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertPlainTypeRuntimeCounts,
  type PlainTypeRuntimePacket,
} from "../../tools/benchmarks/scripts/patina-plain-type-runtime-counts.ts";

const failure: PlainTypeRuntimePacket = {
  case: "unavailable",
  actual: {
    error_count: 1,
    warning_count: 0,
    diagnostics: [{ severity: "error", rule_name: "type/corsa-runtime" }],
  },
};

test("each requested unavailable provider is exactly one error, with no warning fallback", () => {
  assertPlainTypeRuntimeCounts(failure);
  for (const mutate of [
    (packet: typeof failure) => ({ ...packet, actual: { ...packet.actual, error_count: 0 } }),
    (packet: typeof failure) => ({ ...packet, actual: { ...packet.actual, warning_count: 1 } }),
    (packet: typeof failure) => ({ ...packet, actual: { ...packet.actual, diagnostics: [] } }),
    (packet: typeof failure) => ({
      ...packet,
      actual: {
        ...packet.actual,
        diagnostics: [{ severity: "warning", rule_name: "type/corsa-runtime" }],
      },
    }),
    (packet: typeof failure) => ({
      ...packet,
      actual: {
        ...packet.actual,
        diagnostics: [{ severity: "error", rule_name: "type/no-floating-promises" }],
      },
    }),
  ])
    assert.throws(() => assertPlainTypeRuntimeCounts(mutate(failure)));
});

test("ordinary type-aware findings remain warnings and all excluded cells remain empty", () => {
  assertPlainTypeRuntimeCounts({
    case: "active",
    actual: {
      error_count: 0,
      warning_count: 1,
      diagnostics: [{ severity: "warning", rule_name: "type/no-floating-promises" }],
    },
  });
  for (const name of [
    "javascript",
    "declaration-ts",
    "declaration-mts",
    "declaration-cts",
    "default",
    "host-disabled",
    "rule-disabled",
    "no-query",
  ]) {
    const packet = { case: name, actual: { error_count: 0, warning_count: 0, diagnostics: [] } };
    assertPlainTypeRuntimeCounts(packet);
    assert.throws(() => assertPlainTypeRuntimeCounts({ ...packet, actual: failure.actual }));
  }
  assert.throws(() => assertPlainTypeRuntimeCounts({ case: "unknown", actual: failure.actual }));
});
