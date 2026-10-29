import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { compareLspMarkerScans } from "../../tools/support/compat/davinci/lsp-marker-scan-compare.ts";
import { lspMarkerProductionPaths } from "../../tools/support/compat/davinci/lsp-marker-scan-source.ts";

test("paired marker evidence rejects wrong binaries, missing files and changed diagnostics", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-marker-compare-"));
  const write = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(directory, name), JSON.stringify(value));
  const build = {
    features: ["default"],
    rustc: "same rustc",
    cargo: "same cargo",
    target: "same target",
    profile_manifest: ["same profile"],
  };
  const identity = (role: "base" | "head", profile: boolean) => ({
    source: { revision: role === "base" ? "a".repeat(40) : "b".repeat(40) },
    build: { ...build, features: profile ? ["default", "profiling"] : ["default"] },
    binary: {
      sha256: (role === "base" ? (profile ? "e" : "c") : profile ? "f" : "d").repeat(64),
      bytes: 100,
    },
  });
  const capture = (role: "base" | "head", profile: boolean, milliseconds: number) => ({
    complete: true,
    completed_runs: 1,
    runs: 1,
    workspace_files: 2,
    unique_imported_providers: 2,
    server_sha256: identity(role, profile).binary.sha256,
    per_run: [
      {
        corpus: {
          sha256: "1".repeat(64),
          sfcFiles: 2,
          entries: ["App.vue", "Peer.vue", "tsconfig.json", "vite.config.mjs"],
          backendSha256: "2".repeat(64),
          vuePackageSha256: "3".repeat(64),
        },
        opened_files: 2,
        metadata_requests: 2,
        successful_unique_provider_requests: 2,
        resident_document_witnesses: 2,
        body_edit_churn: { edits: 32, unchanged_prop_completion_replies: 32 },
        controlled_edits: [
          {
            diagnostic: {
              code: 2322,
              message: "exact authored diagnostic",
              range: { start: { line: 2, character: 4 } },
            },
            elapsed_ms: milliseconds,
          },
        ],
        warming_elapsed_ms: milliseconds,
      },
    ],
  });
  const productionReceipt = (changed: boolean) => ({
    head: "b".repeat(40),
    parents: ["a".repeat(40), "d".repeat(40)],
    requestedBase: "a".repeat(40),
    expectedPrHead: "d".repeat(40),
    selectedBase: "a".repeat(40),
    eventBaseMatchesActualParent: true,
    policy: "actual checked-out merge first parent",
    productionChanged: changed,
    productionFiles: changed ? [lspMarkerProductionPaths[0]] : [],
    productionDiff: {
      selectedBase: "a".repeat(40),
      head: "b".repeat(40),
      paths: lspMarkerProductionPaths,
    },
  });
  try {
    write("source-selection.json", productionReceipt(true));
    for (const role of ["base", "head"] as const) {
      write(`${role}-normal-identity.json`, identity(role, false));
      write(`${role}-profile-identity.json`, identity(role, true));
      for (let round = 1; round <= 3; round++)
        write(`${role}-normal-${round}.json`, capture(role, false, role === "base" ? 100 : 70));
      write(`${role}-profile.json`, capture(role, true, 200));
      fs.mkdirSync(path.join(directory, `profile-${role}`));
      write(`profile-${role}/run-1.json`, {
        spans: [
          {
            key: "maestro.project.document_root@actual-source.rs:7",
            count: 2,
            wall_ns: { total: 100, self: 100 },
            alloc: {
              calls: 100,
              bytes: 1000,
              self_calls: role === "base" ? 100 : 50,
              self_bytes: role === "base" ? 1000 : 500,
            },
          },
        ],
        micro: [
          ...(role === "base"
            ? [{ name: "fs.read_dir.calls", calls: 2, units: 2 }]
            : [
                { name: "fs.directory_scan.calls", calls: 2, units: 2 },
                { name: "fs.getdents64.calls", calls: 4, units: 4 },
                { name: "fs.getdents64.bytes", calls: 2, units: 4096 },
              ]),
          { name: "fs.read_dir.entries", calls: 100, units: 100 },
        ],
        allocation: { calls: 1000, requested_bytes: 10000 },
      });
    }
    const report = compareLspMarkerScans(directory);
    assert.equal(report.production.comparisonMode, "strict hotspot improvement");
    assert.equal(report.normal.headMedianMs, 70);
    assert.equal(report.profiles.head.calls, 2);
    assert.deepEqual(report.profiles.head.operations, {
      directoryScanCounter: "fs.directory_scan.calls",
      standardReadDirCalls: 0,
      getdents64Calls: 4,
      getdents64Bytes: 4096,
    });
    assert.equal(report.profiles.base.operations.standardReadDirCalls, 2);
    assert.deepEqual(report.profiles.base.root_allocation, {
      invocations: 2,
      total: { self_calls: 100, self_bytes: 1000 },
      per_invocation: { self_calls: 50, self_bytes: 500 },
    });
    assert.deepEqual(report.profiles.head.root_allocation, {
      invocations: 2,
      total: { self_calls: 50, self_bytes: 500 },
      per_invocation: { self_calls: 25, self_bytes: 250 },
    });
    const profilePath = path.join(directory, "profile-head/run-1.json");
    const originalProfile = JSON.parse(fs.readFileSync(profilePath, "utf8")) as {
      spans: { count: number; alloc: { self_calls: number; self_bytes: number } }[];
      micro: { name: string; calls: number; units: number }[];
    };
    const moreTotalWork = structuredClone(originalProfile);
    moreTotalWork.spans[0].count = 4;
    moreTotalWork.spans[0].alloc.self_calls = 120;
    moreTotalWork.spans[0].alloc.self_bytes = 1200;
    write("profile-head/run-1.json", moreTotalWork);
    assert.throws(
      () => compareLspMarkerScans(directory),
      /total allocation hotspot calls must decrease/u,
    );
    const equalTotalWork = structuredClone(originalProfile);
    equalTotalWork.spans[0].alloc.self_calls = 100;
    equalTotalWork.spans[0].alloc.self_bytes = 1000;
    write("profile-head/run-1.json", equalTotalWork);
    assert.throws(
      () => compareLspMarkerScans(directory),
      /total allocation hotspot calls must decrease/u,
    );
    const worsePerInvocation = structuredClone(originalProfile);
    worsePerInvocation.spans[0].count = 1;
    worsePerInvocation.spans[0].alloc.self_calls = 60;
    worsePerInvocation.spans[0].alloc.self_bytes = 600;
    write("profile-head/run-1.json", worsePerInvocation);
    assert.throws(
      () => compareLspMarkerScans(directory),
      /per-invocation allocation hotspot calls must decrease/u,
    );
    write("profile-head/run-1.json", originalProfile);
    const standardReader = structuredClone(originalProfile);
    standardReader.micro = [
      { name: "fs.read_dir.calls", calls: 2, units: 2 },
      { name: "fs.read_dir.entries", calls: 100, units: 100 },
    ];
    write("profile-head/run-1.json", standardReader);
    assert.throws(() => compareLspMarkerScans(directory), /native Linux path/u);
    write("profile-head/run-1.json", originalProfile);
    const badBinary = identity("head", false);
    badBinary.binary.sha256 = identity("base", false).binary.sha256;
    write("head-normal-identity.json", badBinary);
    assert.throws(() => compareLspMarkerScans(directory), /distinct actual measured binaries/u);
    write("head-normal-identity.json", identity("head", false));
    const wrongProfile = identity("head", true);
    wrongProfile.binary.sha256 = identity("head", false).binary.sha256;
    const wrongCapture = capture("head", true, 200);
    wrongCapture.server_sha256 = wrongProfile.binary.sha256;
    write("head-profile-identity.json", wrongProfile);
    write("head-profile.json", wrongCapture);
    assert.throws(
      () => compareLspMarkerScans(directory),
      /instrumented and ordinary executables must differ/u,
    );
    write("head-profile-identity.json", identity("head", true));
    write("head-profile.json", capture("head", true, 200));
    for (const mutate of [
      (value: ReturnType<typeof capture>) => {
        value.per_run[0].resident_document_witnesses--;
      },
      (value: ReturnType<typeof capture>) => {
        value.per_run[0].corpus.sha256 = "wrong source bytes";
      },
      (value: ReturnType<typeof capture>) => {
        value.per_run[0].controlled_edits[0].diagnostic.message = "wrong diagnostic";
      },
      (value: ReturnType<typeof capture>) => {
        value.complete = false;
      },
      (value: ReturnType<typeof capture>) => {
        value.server_sha256 = "wrong measured binary";
      },
    ]) {
      const changed = capture("head", false, 70);
      mutate(changed);
      write("head-normal-2.json", changed);
      assert.throws(() => compareLspMarkerScans(directory), { code: "ERR_ASSERTION" });
      write("head-normal-2.json", capture("head", false, 70));
    }

    const baseProfilePath = path.join(directory, "profile-base/run-1.json");
    const unchangedBase = JSON.parse(
      fs.readFileSync(baseProfilePath, "utf8"),
    ) as typeof originalProfile;
    unchangedBase.spans[0].alloc.self_calls = 50;
    unchangedBase.spans[0].alloc.self_bytes = 500;
    unchangedBase.micro = structuredClone(originalProfile.micro);
    write("profile-base/run-1.json", unchangedBase);
    write("profile-head/run-1.json", originalProfile);
    write("source-selection.json", productionReceipt(false));
    const identicalHeadNormal = identity("head", false);
    identicalHeadNormal.binary.sha256 = identity("base", false).binary.sha256;
    write("head-normal-identity.json", identicalHeadNormal);
    for (let round = 1; round <= 3; round++) {
      const identicalCapture = capture("head", false, 70);
      identicalCapture.server_sha256 = identicalHeadNormal.binary.sha256;
      write(`head-normal-${round}.json`, identicalCapture);
    }
    const identicalHeadProfile = identity("head", true);
    identicalHeadProfile.binary.sha256 = identity("base", true).binary.sha256;
    write("head-profile-identity.json", identicalHeadProfile);
    const identicalProfileCapture = capture("head", true, 200);
    identicalProfileCapture.server_sha256 = identicalHeadProfile.binary.sha256;
    write("head-profile.json", identicalProfileCapture);
    const unchanged = compareLspMarkerScans(directory);
    assert.equal(unchanged.production.comparisonMode, "unchanged native per-invocation parity");
    assert.deepEqual(unchanged.production.totalDrift, {
      invocations: 0,
      self_calls: 0,
      self_bytes: 0,
    });

    const driftedHead = structuredClone(originalProfile);
    driftedHead.spans[0].count = 3;
    driftedHead.spans[0].alloc.self_calls = 75;
    driftedHead.spans[0].alloc.self_bytes = 750;
    write("profile-head/run-1.json", driftedHead);
    assert.deepEqual(compareLspMarkerScans(directory).production.totalDrift, {
      invocations: 1,
      self_calls: 25,
      self_bytes: 250,
    });

    const tampered = productionReceipt(false);
    tampered.productionFiles = [lspMarkerProductionPaths[0]];
    write("source-selection.json", tampered);
    assert.throws(() => compareLspMarkerScans(directory), /production change flag disagrees/u);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
