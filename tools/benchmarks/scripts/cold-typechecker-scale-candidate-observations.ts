/** Exact private allocation/census records. These are not heap/RSS or native-liveness proofs. */
import assert from "node:assert/strict";
import path from "node:path";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { spawnSync, type SpawnSyncOptionsWithBufferEncoding } from "node:child_process";
import type { ReplayOutcome } from "./cold-typechecker-scale-native-custody.ts";
import { classifyRoutePopulation } from "./cold-typechecker-scale-candidate-law.ts";
type Row = Record<string, unknown>;
const object = (value: unknown): Row => {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
const keys = (value: Row, names: string[]) =>
  assert.deepEqual(Object.keys(value).sort(), [...names].sort());
const unsignedNames = [
  "alloc_calls",
  "alloc_zeroed_calls",
  "alloc_failures",
  "alloc_zeroed_failures",
  "alloc_bytes",
  "alloc_zeroed_bytes",
  "dealloc_calls",
  "dealloc_bytes",
  "realloc_calls",
  "realloc_failures",
  "realloc_old_bytes",
  "realloc_new_bytes",
];
const decimal = (value: unknown) => {
  assert.equal(typeof value, "string");
  assert.match(value as string, /^(?:0|[1-9]\d*)$/u);
  const number = BigInt(value as string);
  assert(number < 1n << 64n);
  return number;
};
function counterState(value: unknown) {
  const row = object(value);
  keys(row, [...unsignedNames, "requested_bytes", "released_bytes", "net_bytes_signed"]);
  const numbers = Object.fromEntries(unsignedNames.map((key) => [key, decimal(row[key])]));
  const saturate = (number: bigint) => (number >= 1n << 64n ? (1n << 64n) - 1n : number);
  const requested = saturate(
    numbers.alloc_bytes + numbers.alloc_zeroed_bytes + numbers.realloc_new_bytes,
  );
  const released = saturate(numbers.dealloc_bytes + numbers.realloc_old_bytes);
  assert.equal(decimal(row.requested_bytes), requested);
  assert.equal(decimal(row.released_bytes), released);
  assert.equal(row.net_bytes_signed, (requested - released).toString());
  return row;
}
const bytes = (value: unknown) => {
  assert(
    Array.isArray(value) &&
      value.every((byte) => Number.isInteger(byte) && byte >= 0 && byte <= 255),
  );
  return Buffer.from(value);
};
function exitStatus(value: unknown) {
  const row = object(value);
  keys(row, ["success", "code", "display", "signal"]);
  assert(
    row.code === null ||
      (Number.isSafeInteger(row.code) && Number(row.code) >= 0 && Number(row.code) <= 255),
  );
  assert(row.signal === null || (Number.isSafeInteger(row.signal) && Number(row.signal) > 0));
  assert(row.code === null || row.signal === null);
  assert.equal(row.success, row.code === 0 && row.signal === null);
  assert.equal(typeof row.display, "string");
  return row;
}

export function verifyChildAllocationObservation(
  value: unknown,
  expected: {
    vizePid: number;
    carrierPid: number;
    nativePid: number;
    status: number;
    stdout: Buffer;
    stderr: Buffer;
  },
) {
  const row = object(value);
  keys(row, [
    "schema",
    "process_id",
    "child_id",
    "pre_spawn",
    "bracket_before",
    "while_running",
    "bracket_after",
    "after_wait",
    "result_stage",
    "command_error",
    "status",
    "stdout",
    "stderr",
    "scope",
  ]);
  assert.equal(row.schema, "vize.childAllocationWindow.v1");
  for (const pid of [expected.vizePid, expected.carrierPid, expected.nativePid])
    assert(Number.isSafeInteger(pid) && pid > 0);
  assert.equal(new Set([expected.vizePid, expected.carrierPid, expected.nativePid]).size, 3);
  assert.equal(row.process_id, expected.vizePid);
  assert.equal(row.child_id, expected.carrierPid);
  assert.equal(row.result_stage, "completed");
  assert.equal(row.command_error, null);
  const status = exitStatus(row.status);
  assert.equal(status.code, expected.status);
  assert.equal(status.signal, null);
  assert.equal(status.success, expected.status === 0);
  assert.equal(typeof status.display, "string");
  assert(bytes(row.stdout).equals(expected.stdout), "whole carrier/native stdout differs");
  assert(bytes(row.stderr).equals(expected.stderr), "whole carrier/native stderr differs");
  counterState(row.pre_spawn);
  counterState(row.after_wait);
  const probe = (value: unknown, stage: string) => {
    const p = object(value);
    keys(p, ["state", "status", "error"]);
    assert(["running", "exited", "error"].includes(String(p.state)));
    if (p.state === "running") assert(p.status === null && p.error === null);
    else if (p.state === "exited") {
      assert(p.error === null);
      exitStatus(p.status);
    } else {
      assert.equal(p.status, null);
      const error = object(p.error);
      keys(error, ["stage", "kind", "raw_os_error", "message"]);
      assert.equal(error.stage, stage);
      assert.equal(typeof error.kind, "string");
      assert.equal(typeof error.message, "string");
      assert(error.raw_os_error === null || Number.isSafeInteger(error.raw_os_error));
    }
    return p;
  };
  const before = probe(row.bracket_before, "try_wait_before");
  if (before.state === "running") {
    const after = probe(row.bracket_after, "try_wait_after");
    if (after.state === "running") counterState(row.while_running);
    else assert.equal(row.while_running, null);
  } else {
    assert.equal(row.bracket_after, null);
    assert.equal(row.while_running, null);
  }
  assert.equal(typeof row.scope, "string");
  return {
    raw: row,
    vizePid: expected.vizePid,
    observedCarrierPid: expected.carrierPid,
    separatelyObservedNativePid: expected.nativePid,
    whileCarrierRunningAvailable: row.while_running !== null,
    nativeAtSnapshot:
      "unobserved; immediate carrier bracket does not prove the forwarded Go process has started",
    balanceScope:
      "signed tracked-window requested minus released; independent relaxed loads; not retained/live/usable heap, allocator caches or RSS",
  };
}

export function verifyRouteCensus(value: unknown, programRoot: string) {
  const row = object(value);
  keys(row, [
    "schema",
    "program_root",
    "bindings",
    "positive_bindings",
    "negative_bindings",
    "binding_inline_initialized_bytes",
    "routes",
    "binding_invalidation_path_lengths",
    "binding_invalidation_path_capacities",
    "shared_invalidation_arc_slots",
    "binding_population",
    "positive_route_population",
    "outer_vec_capacity",
    "scope",
  ]);
  assert.equal(row.schema, "vize.routeHandoffMemoryObservation.v2");
  assert(path.isAbsolute(programRoot));
  assert.equal(row.program_root, programRoot);
  for (const key of [
    "bindings",
    "positive_bindings",
    "negative_bindings",
    "binding_inline_initialized_bytes",
    "binding_invalidation_path_lengths",
    "binding_invalidation_path_capacities",
    "shared_invalidation_arc_slots",
  ])
    assert(Number.isSafeInteger(row[key]) && Number(row[key]) >= 0);
  const routes = object(row.routes);
  keys(routes, [
    "route_nodes",
    "source_paths",
    "dependency_paths",
    "source_targets",
    "nested_routes",
    "path_buffer_length_bytes",
    "path_buffer_capacity_bytes",
    "route_vector_initialized_bytes",
    "route_vector_capacity_bytes",
    "package_name_length_bytes_excluded_from_totals",
  ]);
  for (const number of Object.values(routes))
    assert(Number.isSafeInteger(number) && Number(number) >= 0);
  assert(Number(routes.path_buffer_capacity_bytes) >= Number(routes.path_buffer_length_bytes));
  assert(
    Number(routes.route_vector_capacity_bytes) >= Number(routes.route_vector_initialized_bytes),
  );
  assert(
    Number(row.binding_invalidation_path_capacities) >=
      Number(row.binding_invalidation_path_lengths),
  );
  const population = classifyRoutePopulation({
    bindings: Number(row.bindings),
    positiveBindings: Number(row.positive_bindings),
    negativeBindings: Number(row.negative_bindings),
    bindingInlineInitializedBytes: Number(row.binding_inline_initialized_bytes),
  });
  assert.equal(row.binding_population, population.bindings === 0 ? "empty" : "nonempty");
  assert.equal(
    row.positive_route_population,
    population.positiveBindings === 0 ? "none" : "present",
  );
  assert.equal(row.outer_vec_capacity, null);
  assert.equal(typeof row.scope, "string");
  return {
    raw: row,
    population,
    eligibleInitializedBindingWork: population.initializedBindingCloneWorkPresent,
  };
}

/** Preserve the complete report; the existing profiling feature exposes all operation fields. */
export function verifyCandidateAllocationProfile(value: unknown) {
  const report = object(value);
  assert.equal(report.schema_version, 1);
  assert.equal(report.tool, "vize");
  assert.equal(report.command, "check");
  assert.deepEqual(report.truncation, { dropped_spans: 0, dropped_counters: 0 });
  assert(Array.isArray(report.spans) && Array.isArray(report.counters));
  const commands = report.spans.map(object).filter((row) => row.key === "canon.corsa.cli.command");
  assert.equal(commands.length, 1);
  assert.equal(commands[0].count, 1);
  assert(Number(object(commands[0].wall_ns).total) > 0);
  const allocation = object(report.allocation),
    operations = object(allocation.operations);
  keys(allocation, ["operations", "calls", "requested_bytes", "released_bytes", "failures"]);
  keys(operations, unsignedNames);
  for (const number of [
    ...Object.values(operations),
    allocation.calls,
    allocation.requested_bytes,
    allocation.released_bytes,
    allocation.failures,
  ])
    assert(
      Number.isSafeInteger(number) && Number(number) >= 0,
      "allocation JSON integer precision/refusal",
    );
  const n = (key: string) => BigInt(operations[key] as number);
  assert.equal(
    BigInt(allocation.calls as number),
    n("alloc_calls") + n("alloc_zeroed_calls") + n("realloc_calls"),
  );
  assert.equal(
    BigInt(allocation.requested_bytes as number),
    n("alloc_bytes") + n("alloc_zeroed_bytes") + n("realloc_new_bytes"),
  );
  assert.equal(
    BigInt(allocation.released_bytes as number),
    n("dealloc_bytes") + n("realloc_old_bytes"),
  );
  assert.equal(
    BigInt(allocation.failures as number),
    n("alloc_failures") + n("alloc_zeroed_failures") + n("realloc_failures"),
  );
  assert.equal(
    allocation.failures,
    0,
    "retain failed allocation profile, refuse performance comparison",
  );
  return {
    raw: report,
    recordedWindowNetBytes: (
      BigInt(allocation.requested_bytes as number) - BigInt(allocation.released_bytes as number)
    ).toString(),
    scope:
      "whole instrumented report including observer overhead; not RSS, retained heap or a normal timing sample",
  };
}

/** Raw-first untimed capture, including the actual parent PID needed by the private counter join. */
export function captureCandidateUntimed(
  spec: {
    directory: string;
    executable: string;
    imageSha256: string;
    args: string[];
    cwd: string;
    env: NodeJS.ProcessEnv;
  },
  execute: (
    program: string,
    args: string[],
    options: SpawnSyncOptionsWithBufferEncoding,
  ) => ReplayOutcome = spawnSync,
) {
  assert(
    path.isAbsolute(spec.directory) &&
      path.isAbsolute(spec.executable) &&
      path.isAbsolute(spec.cwd),
  );
  fs.mkdirSync(spec.directory); // Fresh attempt only. Never replace prior evidence.
  const write = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(spec.directory, name), JSON.stringify(value, null, 2) + "\n", {
      flag: "wx",
    });
  write("request.json", {
    executable: spec.executable,
    imageSha256: spec.imageSha256,
    args: spec.args,
    cwd: spec.cwd,
    timingEligible: false,
    environment: Object.fromEntries(
      [
        "PATH",
        "GOMAXPROCS",
        "GOGC",
        "GOMEMLIMIT",
        "RAYON_NUM_THREADS",
        "NODE_OPTIONS",
        "NODE_PATH",
        "XDG_CACHE_HOME",
        "TMPDIR",
        "VIZE_CHILD_ALLOCATION_OBSERVATION",
        "VIZE_ROUTE_MEMORY_OBSERVATION",
      ].map((key) => [key, spec.env[key] ?? null]),
    ),
  });
  const digest = () => createHash("sha256").update(fs.readFileSync(spec.executable)).digest("hex");
  assert.match(spec.imageSha256, /^[a-f0-9]{64}$/u);
  assert.equal(fs.realpathSync(spec.executable), spec.executable);
  assert.equal(digest(), spec.imageSha256);
  const started = process.hrtime.bigint();
  const result = execute(spec.executable, spec.args, {
    cwd: spec.cwd,
    env: spec.env,
    timeout: 90_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const streams: Record<string, unknown> = {};
  for (const name of ["stdout", "stderr"] as const) {
    const bytes = result[name];
    streams[name] = Buffer.isBuffer(bytes)
      ? {
          available: true,
          bytes: bytes.length,
          sha256: createHash("sha256").update(bytes).digest("hex"),
        }
      : { available: false, bytes: null, sha256: null };
    if (Buffer.isBuffer(bytes))
      fs.writeFileSync(path.join(spec.directory, name + ".bin"), bytes, { flag: "wx" });
  }
  write("process.json", {
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    invokedPid: result.pid ?? null,
    elapsedNs: (process.hrtime.bigint() - started).toString(),
    streams,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert(Number.isSafeInteger(result.pid) && Number(result.pid) > 0);
  assert(Buffer.isBuffer(result.stdout) && Buffer.isBuffer(result.stderr));
  assert.equal(digest(), spec.imageSha256, "untimed image changed after raw retention");
  return result;
}
