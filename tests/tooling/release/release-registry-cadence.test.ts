import assert from "node:assert/strict";
import { test } from "node:test";
import {
  cadenceRegistryDecision,
  incompletePriorReleaseTargets,
  laggingPriorReleaseCrates,
  priorReleaseCompleteness,
} from "../../../tools/support/release/release/main.ts";

test("a new release requires the prior tag on npm", () => {
  assert.deepEqual(incompletePriorReleaseTargets("0.510.0", ["0.510.0"]), []);
  assert.deepEqual(incompletePriorReleaseTargets("0.510.0", ["0.509.0"]), ["npm:vize@0.510.0"]);
});

test("crates.io gaps of the prior tag are reported for every published crate", async () => {
  let calls = 0;
  const lagging = await laggingPriorReleaseCrates("0.510.0", async (crate) => {
    calls += 1;
    if (crate === "vize_oxc_parser") return { kind: "missing" };
    if (crate === "vize_patina") return { kind: "query-failed", message: "HTTP 429" };
    return { kind: "published" };
  });
  assert.equal(calls, 28);
  assert.deepEqual(lagging, ["crates:vize_oxc_parser@0.510.0", "crates:vize_patina@0.510.0"]);
});

test("cadence waits for npm but not for a stalled crates.io index", async () => {
  const stalledIndex = await priorReleaseCompleteness(
    ["v0.509.0", "v0.510.0"],
    ["0.510.0"],
    async (crate) => (crate === "vize_patina" ? { kind: "missing" } : { kind: "published" }),
  );
  assert.deepEqual(stalledIndex.lagging, ["crates:vize_patina@0.510.0"]);
  assert.equal(cadenceRegistryDecision(stalledIndex), null);

  const incomplete = await priorReleaseCompleteness(
    ["v0.509.0", "v0.510.0"],
    ["0.509.0"],
    async () => ({ kind: "published" }),
  );
  assert.deepEqual(cadenceRegistryDecision(incomplete), {
    release: false,
    reason: "v0.510.0 is incomplete in public registries",
  });
  assert.deepEqual(cadenceRegistryDecision(null), {
    release: false,
    reason: "release registry preflight unavailable",
  });

  assert.equal(
    cadenceRegistryDecision(
      await priorReleaseCompleteness(["v0.510.0"], ["0.510.0"], async () => ({
        kind: "published",
      })),
    ),
    null,
  );
});
