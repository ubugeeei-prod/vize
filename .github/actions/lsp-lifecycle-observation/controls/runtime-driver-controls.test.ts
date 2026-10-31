import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { originalArguments, prepareCommandDirectory, profileLaunchMetadata, selectedSource, sources, validateChurn, validateProfile, validateSourceAuthority } from "../runtime-driver.ts";

const source = selectedSource(process.env["ROLE"]);

type Packet = Record<string, unknown>;
function object(value: unknown): Packet {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
// Synthetic metadata exercises admission/refusal only; it grants no native credit.
function packet(): Packet {
  return { commit: source, status: "failed",
    peakRssKiB: 140432, peakProcessTreeRssKiB: 260376,
    budget: { scale: 1, maxPeakRssMiB: 136, cyclesPerPhase: 20 },
    timingsMs: { cycle: Array(40).fill(1), fileLifecycle: Array(20).fill(1) },
    cycleStats: { cycles: 40 }, rssSamples: ["coldOpen",
      ...Array.from({ length: 20 }, (_, index: number) => `A${index}`),
      ...Array.from({ length: 20 }, (_, index: number) => `B${index}`), "cancellation", "closed",
      ...Array.from({ length: 20 }, (_, index: number) => `file-lifecycle-${index}`)].map((label) => ({ label })) };
}
test("unchanged original task arguments contain no observer RPC or workload substitution", () => {
  assert.notEqual(selectedSource("full"), selectedSource("demand"));
  for (const role of ["full", "demand"] as const) {
    assert.equal(selectedSource(role), sources[role]);
    assert.equal(validateSourceAuthority(role, sources[role], sources[role]), sources[role]);
    const foreign = sources[role === "full" ? "demand" : "full"];
    assert.throws(() => validateSourceAuthority(role, foreign, sources[role]));
    assert.throws(() => validateSourceAuthority(role, sources[role], foreign));
    assert.throws(() => validateSourceAuthority(role, undefined, sources[role]));
  }
  for (const role of [undefined, "", "base", "candidate", "fresh", "reuse", "allocation-observation", "__proto__"])
    assert.throws(() => selectedSource(role));
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
    assert.equal(fs.statSync(path.join(observed, "profile")).mode & 0o777, 0o700);
    assert.throws(() => prepareCommandDirectory(observed, true));
    const incremental = path.join(root, "incremental"); prepareCommandDirectory(incremental, false);
    assert.equal(fs.existsSync(path.join(incremental, "proc")), false);
    assert.equal(fs.existsSync(path.join(incremental, "profile")), false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

const fields = ["read_before_ns", "read_after_ns", "alloc_calls", "alloc_zeroed_calls", "alloc_failures", "alloc_zeroed_failures", "alloc_bytes", "alloc_zeroed_bytes", "dealloc_calls", "dealloc_bytes", "realloc_calls", "realloc_failures", "realloc_old_bytes", "realloc_new_bytes"];
function profile(): Packet {
  return { schema_version: 1, command: "lsp", truncation: { dropped_spans: 0, dropped_counters: 0 },
    counters: [1, 2].flatMap((scan) => ["before", "after"].flatMap((edge, index) => fields.map((field, fieldIndex) => {
      const stage = (scan - 1) * 2 + index;
      const value = field === "read_before_ns" ? stage * 1000 : field === "read_after_ns" ? stage * 1000 + 10 : stage * 100 + fieldIndex;
      return { key: `diagnostic.workspace_symbols.request${scan}.${edge}.${field}`, samples: 1, total: value, min: value, max: value };
    }))),
    spans: [1, 2].flatMap((scan) => ["worker", "descriptor", "script", "style", "contract"].map((region) => ({
      key: `diagnostic.workspace_symbols.scan${scan}.${region}`, count: region === "worker" ? 1 : 5,
      wall_ns: { total: 100, self: 70, min: 1, max: 50, p50: 16, p95: 64, p99: 64 },
      alloc: { calls: 10, bytes: 100, self_calls: 3, self_bytes: 30 },
    }))) };
}
function rows(value: unknown): Packet[] { assert(Array.isArray(value)); return value.map(object); }
function changeCounter(value: Packet, key: string, changes: Packet): void {
  const counters = rows(value["counters"]); const counter = counters.find((row) => row["key"] === key); assert(counter !== undefined);
  Object.assign(counter, changes); value["counters"] = counters;
}
function changeSpan(value: Packet, key: string, changes: Packet): void {
  const spans = rows(value["spans"]); const span = spans.find((row) => row["key"] === key); assert(span !== undefined);
  Object.assign(span, changes); value["spans"] = spans;
}
test("complete static profile retains four global snapshots and both five-region requested-volume sets", () => {
  const value = profile(); const before = JSON.stringify(value); const result = validateProfile(value);
  assert.equal(JSON.stringify(value), before); assert.equal(result["complete"], true); assert.equal(result["qualificationCredit"], 0);
  assert.equal(Object.keys(object(result["snapshots"])).length, 56); assert.equal(rows(result["spans"]).length, 10);
  const differences = rows(result["differences"]); assert.equal(differences.length, 2);
  for (const row of differences) {
    assert.equal(row["requestedBytes"], 300); assert.equal(row["releasedBytes"], 200); assert.equal(row["netRequestedMinusReleasedBytes"], 100);
    assert.equal(row["requestedAllocLikeCalls"], 300);
  }
});
test("global requested-minus-released arithmetic can be negative without a live-heap claim", () => {
  const value = profile();
  for (const scan of [1, 2]) for (const [index, edge] of ["before", "after"].entries()) {
    const amount = ((scan - 1) * 2 + index) * 1000;
    changeCounter(value, `diagnostic.workspace_symbols.request${scan}.${edge}.dealloc_bytes`, { total: amount, min: amount, max: amount });
  }
  for (const row of rows(validateProfile(value)["differences"])) assert.equal(row["netRequestedMinusReleasedBytes"], -800);
});
test("wrong report identity, truncation, missing, duplicate, repeated and foreign counters are refused", () => {
  for (const replacement of [{ schema_version: 2 }, { command: "check" }, { truncation: { dropped_spans: 1, dropped_counters: 0 } }, { truncation: { dropped_spans: 0, dropped_counters: 1 } }])
    assert.throws(() => validateProfile({ ...profile(), ...replacement }));
  const missing = profile(); missing["counters"] = rows(missing["counters"]).slice(1); assert.throws(() => validateProfile(missing));
  const duplicate = profile(); const counters = rows(duplicate["counters"]); duplicate["counters"] = [...counters, counters[0]]; assert.throws(() => validateProfile(duplicate));
  const repeated = profile(); changeCounter(repeated, "diagnostic.workspace_symbols.request1.before.alloc_calls", { samples: 2 }); assert.throws(() => validateProfile(repeated));
  const foreign = profile(); changeCounter(foreign, "diagnostic.workspace_symbols.request1.before.alloc_calls", { key: "diagnostic.workspace_symbols.request3.before.alloc_calls" }); assert.throws(() => validateProfile(foreign));
});
test("counter unsafe/nonfinite/negative values, aggregate mismatches and regressions are refused", () => {
  for (const amount of [Number.NaN, Number.POSITIVE_INFINITY, -1, 0.5, Number.MAX_SAFE_INTEGER + 1]) {
    const value = profile(); changeCounter(value, "diagnostic.workspace_symbols.request1.before.alloc_bytes", { total: amount, min: amount, max: amount });
    assert.throws(() => validateProfile(value));
  }
  const aggregate = profile(); changeCounter(aggregate, "diagnostic.workspace_symbols.request1.before.alloc_calls", { max: 100 }); assert.throws(() => validateProfile(aggregate));
  const backwards = profile(); changeCounter(backwards, "diagnostic.workspace_symbols.request1.after.alloc_calls", { total: 0, min: 0, max: 0 }); assert.throws(() => validateProfile(backwards));
  const clock = profile(); changeCounter(clock, "diagnostic.workspace_symbols.request2.before.read_before_ns", { total: 1, min: 1, max: 1 }); assert.throws(() => validateProfile(clock));
});
test("missing/duplicate/foreign spans and repeated or nonallocating workers are refused", () => {
  const missing = profile(); missing["spans"] = rows(missing["spans"]).slice(1); assert.throws(() => validateProfile(missing));
  const duplicate = profile(); const spans = rows(duplicate["spans"]); duplicate["spans"] = [...spans, spans[0]]; assert.throws(() => validateProfile(duplicate));
  for (const changes of [{ count: 2 }, { count: 0 }, { alloc: { calls: 0, bytes: 0, self_calls: 0, self_bytes: 0 } },
    { alloc: { calls: 1, bytes: 1, self_calls: 2, self_bytes: 0 } }, { wall_ns: { total: Number.NaN } }, { key: "diagnostic.workspace_symbols.scan3.worker" }]) {
    const value = profile(); changeSpan(value, "diagnostic.workspace_symbols.scan1.worker", changes); assert.throws(() => validateProfile(value));
  }
});
function launch(): Packet {
  return { directory: "/synthetic/observed", server: "/synthetic/ci/vize", wrapper: "/synthetic/observed/.vize-profile-lsp.sh",
    profile: "/synthetic/observed/profile/lsp.json", helperSource: "tools/support/compat/davinci/lib/lsp-profile-launch.ts",
    helperSha256: "a".repeat(64), wrapperSha256: "b".repeat(64), addedArguments: ["--profile-json", "/synthetic/observed/profile/lsp.json"] };
}
test("profile launcher metadata discloses exact added flag while binding the measured same ELF", () => {
  const value = launch(); const before = JSON.stringify(value); const result = profileLaunchMetadata(value);
  assert.equal(JSON.stringify(value), before); assert.equal(result["measuredServerImage"], value["server"]);
  assert.equal(result["profilingInstrumentation"], true); assert.equal(result["qualificationCredit"], 0);
  assert.deepEqual(result["addedArguments"], ["--profile-json", "/synthetic/observed/profile/lsp.json"]);
});
test("launcher foreign paths/helpers/hash/extra-argument mutations are refused without spawning", () => {
  for (const changes of [{ directory: "relative" }, { server: "/native\0bad" }, { wrapper: "/foreign/wrapper" },
    { profile: "/foreign/lsp.json" }, { helperSource: "foreign.ts" }, { helperSha256: "a".repeat(63) },
    { wrapperSha256: "B".repeat(64) }, { addedArguments: [] }, { addedArguments: ["--profile-json", "/synthetic/observed/profile/lsp.json", "--other"] },
    { server: "/synthetic/observed/.vize-profile-lsp.sh" }]) assert.throws(() => profileLaunchMetadata({ ...launch(), ...changes }));
});
