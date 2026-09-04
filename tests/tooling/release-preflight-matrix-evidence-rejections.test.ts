import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertRealProjectMatrixReleaseArtifacts,
  requiredRealProjectMatrixShardCount,
} from "../../legacy-tools/github/release-preflight-matrix-evidence.mjs";
import {
  json,
  mutateDivergence,
  realProjectArtifacts,
  shardEntries,
  typecheckRegistry,
} from "./_helpers/release-preflight-matrix-evidence-fixture.ts";
import { successfulReleaseRun } from "./support/release-preflight.ts";

test("release preflight requires same-corpus coverage, mutation oracle, and preparation linkage", async () => {
  for (const [label, mutate, message] of [
    [
      "coverage mismatch",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.baseline.coverage.baselineVueFilesSha256 = "0".repeat(64)),
        ),
      /same non-empty authored Vue corpus/,
    ],
    [
      "empty shared coverage",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => {
          artifact.baseline.coverage.sharedVueFileCount = 0;
          artifact.baseline.coverage.vizeVueFileCount = 0;
          artifact.baseline.coverage.baselineVueFileCount = 0;
        }),
      /same non-empty authored Vue corpus/,
    ],
    [
      "missing mutation",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => delete artifact.mutationOracle),
      /seeded mutation oracle/,
    ],
    [
      "forged broken mutation state",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[1].sharedCount = 0),
        ),
      /seeded mutation oracle/,
    ],
    [
      "unrestored repaired mutation state",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[2].sourceSha256 = "7".repeat(64)),
        ),
      /seeded mutation oracle/,
    ],
    [
      "unchanged broken mutation source",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) =>
            (artifact.mutationOracle.states[1].sourceSha256 =
              artifact.mutationOracle.states[0].sourceSha256),
        ),
      /seeded mutation oracle/,
    ],
    [
      "unhashed mutation source",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[1].sourceSha256 = "not-a-digest"),
        ),
      /seeded mutation oracle/,
    ],
    [
      "diagnostic-bearing clean mutation state",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[0].sharedCount = 1),
        ),
      /seeded mutation oracle/,
    ],
    [
      "diagnostic-bearing repaired mutation state",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.mutationOracle.states[2].sharedCount = 1),
        ),
      /seeded mutation oracle/,
    ],
    [
      "failed surface verdict",
      (entries: Record<string, string>) => {
        entries["surface-verdict.json"] = json({ status: "failure" });
      },
      /surface verdict is failure/,
    ],
    [
      "wrong shard count",
      (entries: Record<string, string>) => {
        const summary = JSON.parse(entries["summary.json"]);
        summary.command.shardCount = requiredRealProjectMatrixShardCount - 1;
        entries["summary.json"] = json(summary);
      },
      /not exact release evidence/,
    ],
    [
      "missing dependency artifact",
      (entries: Record<string, string>) => {
        const name = Object.keys(entries).find((entry) =>
          entry.endsWith("-typecheck-dependencies.json"),
        );
        if (name == null) throw new Error("No typecheck dependency artifact in fixture entries");
        delete entries[name];
      },
      /typecheck dependency artifact/,
    ],
    [
      "unlinked dependency artifact",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.preparation.payloadSha256 = "0".repeat(64)),
        ),
      /missing dependency preparation linkage/,
    ],
  ] as const) {
    const run = successfulReleaseRun("Real Project Matrix", 520);
    await assert.rejects(
      assertRealProjectMatrixReleaseArtifacts({
        run,
        artifacts: realProjectArtifacts(run),
        registry: typecheckRegistry(),
        // Coverage and the mutation oracle sit inside the parity proof; force
        // strict mode here so each malformed evidence path rejects at source.
        enforceParity: true,
        readArtifactEntries: async (artifact) => {
          const entries = shardEntries(Number(artifact.name.split("-").pop()));
          if (artifact.name === "real-project-matrix-0") mutate(entries);
          return entries;
        },
      }),
      message,
      label,
    );
  }
});
