import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { compareLspMarkerScans } from "../../tools/support/compat/davinci/lsp-marker-scan-compare.ts";
import { readJsonRecord } from "../../tools/support/json-value.ts";

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
  try {
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
          { name: "fs.read_dir.calls", calls: 2, units: 2 },
          { name: "fs.read_dir.entries", calls: 100, units: 100 },
        ],
        allocation: { calls: 1000, requested_bytes: 10000 },
      });
    }
    assert.equal(compareLspMarkerScans(directory).normal.headMedianMs, 70);
    const nativeProfile = readJsonRecord(path.join(directory, "profile-head/run-1.json"));
    nativeProfile.micro = [
      { name: "fs.directory_scan.calls", calls: 2, units: 2 },
      { name: "fs.read_dir.entries", calls: 100, units: 100 },
      { name: "fs.getdents64.calls", calls: 4, units: 4 },
      { name: "fs.getdents64.bytes", calls: 2, units: 4096 },
    ];
    write("profile-head/run-1.json", nativeProfile);
    const nativeComparison = compareLspMarkerScans(directory);
    assert.equal(nativeComparison.profiles.head.calls, 2);
    assert.deepEqual(nativeComparison.profiles.head.operations, {
      directoryScanCounter: "fs.directory_scan.calls",
      standardReadDirCalls: 0,
      getdents64Calls: 4,
      getdents64Bytes: 4096,
    });
    assert.equal(nativeComparison.profiles.base.operations.standardReadDirCalls, 2);
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
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
