import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  verifyChildAllocationObservation,
  verifyRouteCensus,
  captureCandidateUntimed,
  verifyCandidateAllocationProfile,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-observations.ts";

const counters = () => ({
  alloc_calls: "10",
  alloc_zeroed_calls: "1",
  alloc_failures: "0",
  alloc_zeroed_failures: "0",
  alloc_bytes: "100",
  alloc_zeroed_bytes: "10",
  dealloc_calls: "2",
  dealloc_bytes: "140",
  realloc_calls: "1",
  realloc_failures: "0",
  realloc_old_bytes: "20",
  realloc_new_bytes: "30",
  requested_bytes: "140",
  released_bytes: "160",
  net_bytes_signed: "-20",
});
function child() {
  const packet = {
    schema: "vize.childAllocationWindow.v1",
    process_id: 101,
    child_id: 102,
    pre_spawn: counters(),
    bracket_before: { state: "running", status: null, error: null },
    while_running: counters(),
    bracket_after: { state: "running", status: null, error: null },
    after_wait: counters(),
    result_stage: "completed",
    command_error: null,
    status: { success: false, code: 2, display: "exit status: 2", signal: null },
    stdout: [...Buffer.from("whole TS5097\r\n")],
    stderr: [],
    scope: "fixture window counter semantics",
  };
  const expected = {
    vizePid: 101,
    carrierPid: 102,
    nativePid: 103,
    status: 2,
    stdout: Buffer.from(packet.stdout),
    stderr: Buffer.alloc(0),
  };
  return { packet, expected };
}
test("child allocation record joins parent/carrier/native identities and preserves signed window balance only", () => {
  const x = child(),
    saved = JSON.stringify(x.packet);
  const result = verifyChildAllocationObservation(x.packet, x.expected);
  assert.equal(result.whileCarrierRunningAvailable, true);
  assert.equal(result.raw.pre_spawn, x.packet.pre_spawn);
  assert(result.nativeAtSnapshot.startsWith("unobserved"));
  assert.equal(JSON.stringify(x.packet), saved);
  const unavailable = {
    ...x.packet,
    while_running: null,
    bracket_after: { state: "exited", status: x.packet.status, error: null },
  };
  assert.equal(
    verifyChildAllocationObservation(unavailable, x.expected).whileCarrierRunningAvailable,
    false,
  );
});
test("child allocation record refuses wrong IDs/raw/nulls, invented running windows and unsafe counters", () => {
  for (const change of [
    { child_id: 999 },
    { process_id: 999 },
    { stdout: [] },
    { stderr: null },
    { command_error: {} },
    {
      bracket_before: {
        state: "exited",
        status: { success: false, code: 2, display: "exit", signal: null },
        error: null,
      },
    },
    { bracket_after: { state: "exited", status: {}, error: null }, while_running: null },
    { pre_spawn: { ...counters(), requested_bytes: "141" } },
    { pre_spawn: { ...counters(), alloc_bytes: "18446744073709551616" } },
    { after_wait: null },
    { result_stage: "spawn" },
  ]) {
    const x = child();
    assert.throws(() => verifyChildAllocationObservation({ ...x.packet, ...change }, x.expected));
  }
});

function route(bindings: number, positive: number) {
  return {
    schema: "vize.routeHandoffMemoryObservation.v2",
    program_root: "/owned/project",
    bindings,
    positive_bindings: positive,
    negative_bindings: bindings - positive,
    binding_inline_initialized_bytes: bindings * 64,
    routes: {
      route_nodes: positive,
      source_paths: 0,
      dependency_paths: 0,
      source_targets: 0,
      nested_routes: 0,
      path_buffer_length_bytes: 0,
      path_buffer_capacity_bytes: 0,
      route_vector_initialized_bytes: 0,
      route_vector_capacity_bytes: 0,
      package_name_length_bytes_excluded_from_totals: 0,
    },
    binding_invalidation_path_lengths: 0,
    binding_invalidation_path_capacities: 0,
    shared_invalidation_arc_slots: 0,
    binding_population: bindings === 0 ? "empty" : "nonempty",
    positive_route_population: positive === 0 ? "none" : "present",
    outer_vec_capacity: null,
    scope: "fixture structural counts, not allocations",
  };
}
test("route census retains nonempty negative eligibility and unknown outer capacity", () => {
  assert.equal(
    verifyRouteCensus(route(0, 0), "/owned/project").eligibleInitializedBindingWork,
    false,
  );
  const negative = verifyRouteCensus(route(2, 0), "/owned/project");
  assert.equal(negative.eligibleInitializedBindingWork, true);
  assert.equal(negative.population.population, "nonempty-negative-bindings");
  assert.equal(negative.raw.outer_vec_capacity, null);
  assert.equal(
    verifyRouteCensus(route(2, 1), "/owned/project").eligibleInitializedBindingWork,
    true,
  );
});
test("route census refuses wrong population/root, missing fields and inferred capacity", () => {
  for (const change of [
    { bindings: 3 },
    { program_root: "/other" },
    { outer_vec_capacity: 2 },
    { binding_population: "empty" },
    { positive_route_population: "present" },
    { routes: {} },
    { opaque: true },
    { binding_inline_initialized_bytes: Number.MAX_SAFE_INTEGER + 1 },
  ])
    assert.throws(() => verifyRouteCensus({ ...route(2, 0), ...change }, "/owned/project"));
});

test("untimed capture retains real result PID/raw or unavailable streams BEFORE a failed verdict", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "untimed-raw-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const executable = path.join(root, "fake-image");
  fs.writeFileSync(executable, "synthetic metadata image; never executed");
  const imageSha256 = createHash("sha256").update(fs.readFileSync(executable)).digest("hex");
  const spec = (directory: string) => ({
    directory: path.join(root, directory),
    executable,
    imageSha256,
    args: ["check"],
    cwd: root,
    env: {},
  });
  const success = spec("success");
  const result = captureCandidateUntimed(success, () => ({
    status: 0,
    signal: null,
    pid: 123,
    stdout: Buffer.from("whole\r\n"),
    stderr: Buffer.alloc(0),
  }));
  assert.equal(result.pid, 123);
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(success.directory, "process.json"), "utf8")).invokedPid,
    123,
  );
  const failed = spec("failed");
  assert.throws(() =>
    captureCandidateUntimed(failed, () => ({
      status: null,
      signal: null,
      error: new Error("synthetic spawn error"),
      stdout: null,
      stderr: null,
    })),
  );
  const raw = JSON.parse(fs.readFileSync(path.join(failed.directory, "process.json"), "utf8"));
  assert.equal(raw.invokedPid, null);
  assert.deepEqual(raw.streams.stdout, { available: false, bytes: null, sha256: null });
  assert(!fs.existsSync(path.join(failed.directory, "stdout.bin")));
  assert.throws(() =>
    captureCandidateUntimed(success, () => {
      throw new Error("must not execute reused attempt");
    }),
  );
});

test("post-result image failure leaves all raw streams/status before refusing", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "untimed-image-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const executable = path.join(root, "fake-image"),
    directory = path.join(root, "capture");
  fs.writeFileSync(executable, "original image");
  const imageSha256 = createHash("sha256").update(fs.readFileSync(executable)).digest("hex");
  assert.throws(() =>
    captureCandidateUntimed(
      { directory, executable, imageSha256, args: [], cwd: root, env: {} },
      () => {
        fs.writeFileSync(executable, "changed image");
        return {
          status: 0,
          signal: null,
          pid: 123,
          stdout: Buffer.from("original whole output"),
          stderr: Buffer.from("original stderr"),
        };
      },
    ),
  );
  assert.equal(
    fs.readFileSync(path.join(directory, "stdout.bin"), "utf8"),
    "original whole output",
  );
  assert(fs.existsSync(path.join(directory, "process.json")));
});

test("allocation profile retains complete operations/micro/spans and refuses missing/truncated/unsafe totals", () => {
  const sample = () => ({
    schema_version: 1,
    tool: "vize",
    command: "check",
    tool_version: "fixture",
    budget: {},
    truncation: { dropped_spans: 0, dropped_counters: 0 },
    micro: [{ opaque: "preserved" }],
    counters: [],
    spans: [{ key: "canon.corsa.cli.command", count: 1, wall_ns: { total: 1 }, opaque: true }],
    allocation: {
      operations: {
        alloc_calls: 1,
        alloc_zeroed_calls: 2,
        alloc_failures: 0,
        alloc_zeroed_failures: 0,
        alloc_bytes: 100,
        alloc_zeroed_bytes: 20,
        dealloc_calls: 2,
        dealloc_bytes: 70,
        realloc_calls: 3,
        realloc_failures: 0,
        realloc_old_bytes: 10,
        realloc_new_bytes: 30,
      },
      calls: 6,
      requested_bytes: 150,
      released_bytes: 80,
      failures: 0,
    },
  });
  const input = sample(),
    saved = JSON.stringify(input);
  const result = verifyCandidateAllocationProfile(input);
  assert.equal(result.raw, input);
  assert.equal(result.recordedWindowNetBytes, "70");
  assert.equal(JSON.stringify(input), saved);
  for (const alter of [
    (x: ReturnType<typeof sample>) => {
      x.allocation.requested_bytes = 151;
    },
    (x: ReturnType<typeof sample>) => {
      x.allocation.calls = 7;
    },
    (x: ReturnType<typeof sample>) => {
      x.truncation.dropped_spans = 1;
    },
    (x: ReturnType<typeof sample>) => {
      x.spans[0].count = 2;
    },
    (x: ReturnType<typeof sample>) => {
      x.allocation.operations.alloc_bytes = Number.MAX_SAFE_INTEGER + 1;
    },
    (x: ReturnType<typeof sample>) => {
      x.allocation.operations.alloc_failures = 1;
      x.allocation.failures = 1;
    },
  ]) {
    const x = sample();
    alter(x);
    assert.throws(() => verifyCandidateAllocationProfile(x));
  }
  assert.throws(() => verifyCandidateAllocationProfile({ ...input, allocation: null }));
  assert.throws(() =>
    verifyCandidateAllocationProfile({
      ...input,
      allocation: { calls: 6, requested_bytes: 150, released_bytes: 80, failures: 0 },
    }),
  );
});
