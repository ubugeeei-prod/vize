import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { originalArguments, prepareCommandDirectory, validateChurn } from "../runtime-driver.ts";

type Packet = Record<string, unknown>;
function object(value: unknown): Packet {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
// Synthetic metadata exercises admission/refusal only; it grants no native credit.
function packet(): Packet {
  return { commit: "90800904bcfbf3ad073ba95349852ee71ffb37e2", status: "failed",
    peakRssKiB: 140432, peakProcessTreeRssKiB: 260376,
    budget: { scale: 1, maxPeakRssMiB: 136, cyclesPerPhase: 20 },
    timingsMs: { cycle: Array(40).fill(1), fileLifecycle: Array(20).fill(1) },
    cycleStats: { cycles: 40 }, rssSamples: ["coldOpen",
      ...Array.from({ length: 20 }, (_, index: number) => `A${index}`),
      ...Array.from({ length: 20 }, (_, index: number) => `B${index}`), "cancellation", "closed",
      ...Array.from({ length: 20 }, (_, index: number) => `file-lifecycle-${index}`)].map((label) => ({ label })) };
}
test("unchanged original task arguments contain no observer RPC or workload substitution", () => {
  assert.deepEqual(originalArguments("test:performance:lsp-incremental"),
    ["run", "--no-cache", "--filter", "./tests", "test:performance:lsp-incremental"]);
  assert.deepEqual(originalArguments("test:performance:lsp-churn"),
    ["run", "--no-cache", "--filter", "./tests", "test:performance:lsp-churn"]);
});
test("a complete failed original metric remains a failed observation with zero qualification credit", () => {
  const source = packet(); const before = JSON.stringify(source); const result = validateChurn(source);
  assert.equal(JSON.stringify(source), before);
  assert.equal(result["originalStatus"], "failed"); assert.equal(result["qualificationCredit"], 0);
  assert.equal(result["peakRssKiB"], 140432);
});
test("foreign source, raised budget or scale are refused", () => {
  const foreign = packet(); foreign["commit"] = "0".repeat(40); assert.throws(() => validateChurn(foreign));
  for (const [field, value] of [["scale", 2], ["maxPeakRssMiB", 140], ["cyclesPerPhase", 10]]) {
    assert(typeof field === "string"); const changed = packet();
    const budget = object(changed["budget"]); budget[field] = value; changed["budget"] = budget;
    assert.throws(() => validateChurn(changed));
  }
});
test("partial or reordered original observations are refused", () => {
  const partial = packet(); partial["timingsMs"] = { cycle: Array(39).fill(1), fileLifecycle: Array(20).fill(1) };
  assert.throws(() => validateChurn(partial));
  const reordered = packet(); const samples = reordered["rssSamples"]; assert(Array.isArray(samples));
  samples.reverse(); assert.throws(() => validateChurn(reordered));
  const wrongCount = packet(); wrongCount["cycleStats"] = { cycles: 39 }; assert.throws(() => validateChurn(wrongCount));
});

test("observed proc directory is created privately and stale command output is refused", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-observer-directory-"));
  try {
    const observed = path.join(root, "observed"); prepareCommandDirectory(observed, true);
    assert(fs.statSync(path.join(observed, "proc")).isDirectory());
    assert.equal(fs.statSync(path.join(observed, "proc")).mode & 0o777, 0o700);
    assert.throws(() => prepareCommandDirectory(observed, true));
    const incremental = path.join(root, "incremental"); prepareCommandDirectory(incremental, false);
    assert.equal(fs.existsSync(path.join(incremental, "proc")), false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});
