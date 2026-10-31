import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { originalArguments, prepareCommandDirectory, validateIncremental, validateSmapsSummary, VBEN_LANES, MISSKEY_LANES } from "../runtime-driver.ts";
import { SMAPS_LIMITS } from "../smaps-snapshots.ts";
import type { ImageIdentity } from "../types.ts";

type Packet = Record<string, unknown>;
const source = "c34d95586eb9459560c9be4990ff9ebdba3bde61";
function object(value: unknown): Packet {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
const budgets: Record<string, number> = { initialize: 1000, coldOpen: 8000, coldOpenSecondApp: 1500, completion: 1000, hover: 1000, warmNoop: 1500, leafBroken: 1500, leafRepaired: 1500, sharedBroken: 3000, sharedBrokenSecondApp: 3000, sharedRepaired: 3000, sharedRepairedSecondApp: 3000, warmNoopSecondApp: 1500, cancellationConverge: 6000 };
// Synthetic metadata exercises refusal only; no process or native credit.
function packet(suite: "misskey" | "vben" = "vben"): Packet {
  const lanes = suite === "vben" ? VBEN_LANES : MISSKEY_LANES;
  const members = [{ pid: 123, parentPid: 99, role: "root", rssKiB: 155560 },
    { pid: 222, parentPid: 123, role: "descendant", rssKiB: 118272 },
    { pid: 223, parentPid: 123, role: "descendant", rssKiB: 144640 },
    { pid: 224, parentPid: 123, role: "descendant", rssKiB: 111872 }];
  const tree = { totalKiB: 530344, processes: members.length, members };
  return { schemaVersion: 3, commit: source, fixture: suite === "vben" ? "vue-vben-admin/apps+packages+playground" : "misskey/packages/frontend", status: "failed", failure: "original tree gate failed",
    fixtureRevision: suite === "vben" ? "748b182e440863711dc9624924e92440f5ddc4d3" : "810faa8e5db4bbf9d6d408a61df148559340ee99",
    budget: { scale: 1, suite: `${suite}-lsp-incremental`, laneHardTimeoutMs: 60000,
      maxPeakRssMiB: 256, maxPeakProcessTreeRssMiB: suite === "vben" ? 512 : 384,
      maxProcessTreeSize: suite === "vben" ? 4 : 3, laneBudgetsMs: Object.fromEntries(lanes.map(lane => [lane, budgets[lane]])) },
    timingsMs: Object.fromEntries(lanes.map(lane => [lane, 1])), rssSamplesKiB: Object.fromEntries(lanes.map(lane => [lane, 155560])),
    processTreeSamples: Object.fromEntries(lanes.map(lane => [lane, structuredClone(tree)])),
    sampledPeakRssKiB: 155560, sampledPeakProcessTreeRssKiB: 530344, sampledPeakProcessTreeSize: members.length,
    sampledPeakProcessTree: { label: "initialize", ...tree, memberRssSumKiB: 530344 } };
}
test("only the original incremental pair is scheduled, with no churn or observer RPC", () => {
  assert.deepEqual(originalArguments("test:performance:lsp-incremental"),
    ["run", "--no-cache", "--filter", "./tests", "test:performance:lsp-incremental"]);
});
test("complete failed original metrics retain failure and zero credit without mutation", () => {
  for (const suite of ["misskey", "vben"] as const) {
    const value = packet(suite); const before = JSON.stringify(value); const result = validateIncremental(value, suite);
    assert.equal(JSON.stringify(value), before); assert.equal(result["originalStatus"], "failed");
    assert.equal(result["qualificationCredit"], 0); assert.equal(result["sampledPeakProcessTreeRssKiB"], 530344);
  }
});
test("foreign source, suite, raised ceilings and scale are refused", () => {
  for (const changes of [{ commit: "0".repeat(40) }, { fixture: "misskey" }, { fixtureRevision: "0".repeat(40) }, { schemaVersion: 2 }])
    assert.throws(() => validateIncremental({ ...packet(), ...changes }, "vben"));
  for (const [field, value] of [["scale", 2], ["maxPeakRssMiB", 257], ["maxPeakProcessTreeRssMiB", 513], ["maxProcessTreeSize", 5], ["laneHardTimeoutMs", 60001]]) {
    assert(typeof field === "string"); const changed = packet(); const budget = object(changed["budget"]);
    budget[field] = value; changed["budget"] = budget; assert.throws(() => validateIncremental(changed, "vben"));
  }
});
test("missing, reordered, extra or unsafe lanes and inconsistent tree receipts refuse", () => {
  const missing = packet(); const timings = object(missing["timingsMs"]); delete timings["hover"]; missing["timingsMs"] = timings;
  assert.throws(() => validateIncremental(missing, "vben"));
  const reversed = packet(); reversed["timingsMs"] = Object.fromEntries(Object.entries(object(reversed["timingsMs"])).reverse());
  assert.throws(() => validateIncremental(reversed, "vben"));
  const extra = packet(); extra["rssSamplesKiB"] = { ...object(extra["rssSamplesKiB"]), extra: 1 };
  assert.throws(() => validateIncremental(extra, "vben"));
  for (const changes of [{ sampledPeakProcessTreeRssKiB: 530343 }, { sampledPeakProcessTreeSize: 3 }, { sampledPeakRssKiB: Number.NaN }])
    assert.throws(() => validateIncremental({ ...packet(), ...changes }, "vben"));
  const wrongSum = packet(); const trees = object(wrongSum["processTreeSamples"]); const tree = object(trees["initialize"]);
  tree["totalKiB"] = 1; trees["initialize"] = tree; wrongSum["processTreeSamples"] = trees;
  assert.throws(() => validateIncremental(wrongSum, "vben"));
});
test("observed output is private and stale directories refuse", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vben-observer-"));
  try {
    const directory = path.join(root, "observed"); prepareCommandDirectory(directory, true);
    assert.equal(fs.statSync(path.join(directory, "proc")).mode & 0o777, 0o700);
    assert.throws(() => prepareCommandDirectory(directory, true));
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

const server: ImageIdentity = { dev: "1", ino: "2", bytes: 10, mtimeNs: "3", ctimeNs: "4", role: "server", realpath: "/owned/server", sha256: "a".repeat(64) };
const native: ImageIdentity = { ...server, ino: "5", role: "native", realpath: "/owned/native", sha256: "b".repeat(64) };
const target = { pid: 222, birthTicks: "333", parentPid: 123, executableSha256: native.sha256 };
function incompleteSmaps(): Packet {
  const lanes = ["coldOpenSecondApp", "sharedBrokenSecondApp", "sharedRepaired", "sharedRepairedSecondApp"];
  return { schema: "vize.lsp.smaps-snapshots", version: 1, mode: "vben-native", diagnosticOnly: true, qualificationCredit: 0,
    sourceRevision: source, workflowRevision: "c".repeat(40), rootPid: 123, rootBirthTicks: "456", limits: SMAPS_LIMITS,
    ownerRoot: { pid: 123, birthTicks: "456", image: server },
    samplingTarget: { pid: 222, birthTicks: null, selectedBirthTicks: "333", image: native },
    slots: lanes.map(lane => ({ label: `${lane}-settlement-trigger`, triggerPhase: `vben-${lane}-settled`, state: "missing" })),
    snapshotsValid: 0, phaseContained: 0, allSlotsAccounted: false, allReadsComplete: false,
    incomplete: true, inFlight: false, bytesRead: 0, outputBytes: 0, totalVmas: 0 };
}
const validate = (value: unknown): Packet => validateSmapsSummary(value, 123, "456", "c".repeat(40), target, native, server);
test("incomplete native intervals distinguish owner from target without gaining credit", () => {
  const value = incompleteSmaps(); const before = JSON.stringify(value); const result = validate(value);
  assert.equal(JSON.stringify(value), before); assert.equal(result["incomplete"], true);
  assert.equal(result["qualificationCredit"], 0);
});
test("native target, image, owner, source and workflow substitutions refuse", () => {
  for (const changes of [{ sourceRevision: "0".repeat(40) }, { workflowRevision: "d".repeat(40) }, { rootPid: 124 },
    { rootBirthTicks: "457" }, { mode: "server" },
    { samplingTarget: { pid: 223, selectedBirthTicks: "333", birthTicks: null, image: native } },
    { samplingTarget: { pid: 222, selectedBirthTicks: "334", birthTicks: null, image: native } },
    { samplingTarget: { pid: 222, selectedBirthTicks: "333", birthTicks: "334", image: native } },
    { samplingTarget: { pid: 222, selectedBirthTicks: "333", birthTicks: null, image: server } },
    { ownerRoot: { pid: 124, birthTicks: "456", image: server } }])
    assert.throws(() => validate({ ...incompleteSmaps(), ...changes }));
});
test("caps and contradictory completion metadata refuse", () => {
  for (const changes of [{ bytesRead: SMAPS_LIMITS.readBytes + 1 }, { totalVmas: Number.NaN },
    { outputBytes: SMAPS_LIMITS.outputBytes + 1 }, { limits: { ...SMAPS_LIMITS, vmas: 513 } },
    { snapshotsValid: 1 }, { allSlotsAccounted: true }, { allReadsComplete: true }, { phaseContained: 1 }, { incomplete: false }])
    assert.throws(() => validate({ ...incompleteSmaps(), ...changes }));
});
