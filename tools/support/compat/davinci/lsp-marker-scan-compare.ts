import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  type LspMarkerSourceReceipt,
  verifyLspMarkerProductionReceipt,
} from "./lsp-marker-scan-source.ts";

interface Run {
  corpus: {
    sha256: string;
    sfcFiles: number;
    entries: unknown[];
    backendSha256: string;
    vuePackageSha256: string;
  };
  opened_files: number;
  metadata_requests: number;
  successful_unique_provider_requests: number;
  resident_document_witnesses: number;
  body_edit_churn: unknown;
  controlled_edits: Record<string, unknown>[];
  warming_elapsed_ms: number;
}
interface Capture {
  complete: boolean;
  completed_runs: number;
  runs: number;
  workspace_files: number;
  unique_imported_providers: number;
  server_sha256: string;
  per_run: Run[];
}
interface Identity {
  source: { revision: string };
  build: {
    features: string[];
    rustc: string;
    cargo: string;
    target: string;
    profile_manifest: string[];
  };
  binary: { sha256: string; bytes: number };
}
interface SourceSpan {
  count: number;
  wall_ns: { total: number; self: number };
  alloc: { calls: number; bytes: number; self_calls: number; self_bytes: number };
}
interface RootAllocation {
  invocations: number;
  total: { self_calls: number; self_bytes: number };
  per_invocation: { self_calls: number; self_bytes: number };
}
const median = (values: number[]) =>
  [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
const rootAllocation = (root: SourceSpan): RootAllocation => ({
  invocations: root.count,
  total: {
    self_calls: root.alloc.self_calls,
    self_bytes: root.alloc.self_bytes,
  },
  per_invocation: {
    self_calls: root.alloc.self_calls / root.count,
    self_bytes: root.alloc.self_bytes / root.count,
  },
});

/** Reject mismatched executables, inputs, providers or complete diagnostic packets. */
export function compareLspMarkerScans(directory: string) {
  const load = <T>(name: string) =>
    JSON.parse(fs.readFileSync(path.join(directory, name), "utf8")) as T;
  const baseIdentity = load<Identity>("base-normal-identity.json");
  const headIdentity = load<Identity>("head-normal-identity.json");
  const production = verifyLspMarkerProductionReceipt(
    load<LspMarkerSourceReceipt>("source-selection.json"),
    baseIdentity.source.revision,
    headIdentity.source.revision,
  );
  assert.notEqual(baseIdentity.source.revision, headIdentity.source.revision);
  if (production.productionChanged)
    assert.notEqual(
      baseIdentity.binary.sha256,
      headIdentity.binary.sha256,
      "different production code must produce distinct actual measured binaries",
    );
  assert.deepEqual(
    baseIdentity.build,
    headIdentity.build,
    "compiler, features and flags must match",
  );
  assert.ok(
    !baseIdentity.build.features.includes("profiling"),
    "ordinary timing must use the ordinary allocator",
  );
  if (production.productionChanged)
    assert.notEqual(
      load<Identity>("base-profile-identity.json").binary.sha256,
      load<Identity>("head-profile-identity.json").binary.sha256,
      "source profiles must use distinct actual baseline and candidate binaries",
    );
  const baseline = load<Capture>("base-normal-1.json");
  const reference = baseline.per_run[0];
  const semantics = (run: Run) => ({
    churn: run.body_edit_churn,
    edits: run.controlled_edits.map(({ elapsed_ms: _timing, ...semantic }) => semantic),
  });
  const admit = (capture: Capture, binaryHash: string) => {
    assert.equal(capture.complete, true);
    assert.equal(capture.completed_runs, 1);
    assert.equal(capture.runs, 1);
    assert.equal(capture.per_run.length, 1);
    assert.equal(capture.workspace_files, baseline.workspace_files);
    assert.equal(
      capture.server_sha256,
      binaryHash,
      "the harness must observe the built executable",
    );
    const run = capture.per_run[0];
    assert.equal(run.opened_files, capture.workspace_files);
    assert.equal(run.resident_document_witnesses, capture.workspace_files);
    assert.equal(run.metadata_requests, capture.unique_imported_providers);
    assert.equal(run.successful_unique_provider_requests, capture.unique_imported_providers);
    assert.equal(capture.unique_imported_providers, baseline.unique_imported_providers);
    assert.equal(run.corpus.sfcFiles, capture.workspace_files);
    assert.equal(run.corpus.entries.length, capture.workspace_files + 2);
    assert.deepEqual(
      run.corpus,
      reference.corpus,
      "all authored bytes and installed providers must match",
    );
    assert.deepEqual(
      semantics(run),
      semantics(reference),
      "every prop, emit, expose and body diagnostic must match",
    );
    assert.ok(run.warming_elapsed_ms > 0);
    return run.warming_elapsed_ms;
  };
  const normal = { base: [] as number[], head: [] as number[] };
  for (const role of ["base", "head"] as const) {
    const identity = role === "base" ? baseIdentity : headIdentity;
    for (let round = 1; round <= 3; round++) {
      normal[role].push(
        admit(load<Capture>(`${role}-normal-${round}.json`), identity.binary.sha256),
      );
    }
  }
  const profiles = {} as Record<
    "base" | "head",
    {
      root: SourceSpan;
      root_allocation: RootAllocation;
      allocation: unknown;
      calls: number;
      entries: number;
      operations: {
        directoryScanCounter: string;
        standardReadDirCalls: number;
        getdents64Calls: number;
        getdents64Bytes: number;
      };
    }
  >;
  let profileBuild: Identity["build"] | undefined;
  for (const role of ["base", "head"] as const) {
    const identity = load<Identity>(`${role}-profile-identity.json`);
    assert.equal(
      identity.source.revision,
      (role === "base" ? baseIdentity : headIdentity).source.revision,
    );
    assert.ok(identity.build.features.includes("profiling"));
    assert.notEqual(
      identity.binary.sha256,
      (role === "base" ? baseIdentity : headIdentity).binary.sha256,
      "instrumented and ordinary executables must differ",
    );
    if (profileBuild) assert.deepEqual(identity.build, profileBuild);
    profileBuild = identity.build;
    admit(load<Capture>(`${role}-profile.json`), identity.binary.sha256);
    const profile = load<{
      spans: (SourceSpan & { key: string })[];
      micro: { name: string; calls: number; units: number }[];
      allocation: unknown;
    }>(`profile-${role}/run-1.json`);
    const root = profile.spans.find((span) =>
      span.key.startsWith("maestro.project.document_root@"),
    );
    assert.ok(root && root.count > 0 && root.alloc.self_calls > 0 && root.alloc.self_bytes > 0);
    const entryCounter = profile.micro.filter((counter) => counter.name === "fs.read_dir.entries");
    const sum = (name: string) =>
      profile.micro
        .filter((counter) => counter.name === name)
        .reduce((total, counter) => total + counter.units, 0);
    // Earlier sources always called std::fs::read_dir for every scan. New
    // sources distinguish that real API call from a native scan attempt.
    const directoryScanCounter = profile.micro.some(
      (counter) => counter.name === "fs.directory_scan.calls",
    )
      ? "fs.directory_scan.calls"
      : "fs.read_dir.calls";
    const calls = sum(directoryScanCounter);
    const entries = entryCounter.reduce((sum, counter) => sum + counter.units, 0);
    assert.ok(calls > 0 && entries > baseline.workspace_files);
    profiles[role] = {
      root,
      root_allocation: rootAllocation(root),
      allocation: profile.allocation,
      calls,
      entries,
      operations: {
        directoryScanCounter,
        standardReadDirCalls: sum("fs.read_dir.calls"),
        getdents64Calls: sum("fs.getdents64.calls"),
        getdents64Bytes: sum("fs.getdents64.bytes"),
      },
    };
  }
  if (production.productionChanged) {
    assert.ok(
      profiles.head.root_allocation.total.self_calls <
        profiles.base.root_allocation.total.self_calls,
      "the measured total allocation hotspot calls must decrease",
    );
    assert.ok(
      profiles.head.root_allocation.total.self_bytes <
        profiles.base.root_allocation.total.self_bytes,
      "the measured total allocation hotspot bytes must decrease",
    );
    assert.ok(
      profiles.head.root_allocation.per_invocation.self_calls <
        profiles.base.root_allocation.per_invocation.self_calls,
      "the measured per-invocation allocation hotspot calls must decrease",
    );
    assert.ok(
      profiles.head.root_allocation.per_invocation.self_bytes <
        profiles.base.root_allocation.per_invocation.self_bytes,
      "the measured per-invocation allocation hotspot bytes must decrease",
    );
  } else {
    assert.deepEqual(
      profiles.head.root_allocation.per_invocation,
      profiles.base.root_allocation.per_invocation,
      "unchanged marker sources must preserve exact per-invocation allocation",
    );
    assert.ok(
      profiles.base.operations.getdents64Calls > 0 &&
        profiles.base.operations.standardReadDirCalls === 0,
      "the unchanged baseline must scan through the native Linux path",
    );
  }
  assert.ok(
    profiles.head.operations.getdents64Calls > 0 &&
      profiles.head.operations.standardReadDirCalls === 0,
    "the candidate must scan through the native Linux path",
  );
  const totalDrift = {
    invocations:
      profiles.head.root_allocation.invocations - profiles.base.root_allocation.invocations,
    self_calls:
      profiles.head.root_allocation.total.self_calls -
      profiles.base.root_allocation.total.self_calls,
    self_bytes:
      profiles.head.root_allocation.total.self_bytes -
      profiles.base.root_allocation.total.self_bytes,
  };
  return {
    scope:
      "full production LSP residency plus controlled edits; fixture generation/hashing excluded; fresh process per sample",
    files: baseline.workspace_files,
    corpusSha256: reference.corpus.sha256,
    normal: {
      ...normal,
      baseMedianMs: median(normal.base),
      headMedianMs: median(normal.head),
      warmingChangePct: 100 * (median(normal.head) / median(normal.base) - 1),
    },
    profiles,
    production: {
      productionChanged: production.productionChanged,
      productionFiles: production.productionFiles,
      comparisonMode: production.productionChanged
        ? "strict hotspot improvement"
        : "unchanged native per-invocation parity",
      totalDrift,
    },
    note: production.productionChanged
      ? "strict hotspot improvement; profiling counts/cumulative requested bytes are separate from normal timing and process-tree RSS; no canonical 500ms qualification"
      : "unchanged production sources; total drift is recorded without claiming improvement; profiling counts/cumulative requested bytes are separate from normal timing and process-tree RSS",
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const directory = path.resolve(process.argv[2]);
  const report = compareLspMarkerScans(directory);
  fs.writeFileSync(path.join(directory, "comparison.json"), JSON.stringify(report, null, 2) + "\n");
  console.log(JSON.stringify(report, null, 2));
}
