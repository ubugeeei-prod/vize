import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertRealProjectMatrixReleaseArtifacts,
  requireRealProjectMatrixRun,
} from "../../legacy-tools/github/release-preflight-matrix-evidence.mjs";
import {
  mutateDivergence,
  realProjectArtifacts,
  shardEntries,
  typecheckRegistry,
} from "./_helpers/release-preflight-matrix-evidence-fixture.ts";
import { successfulReleaseRun } from "./support/release-preflight.ts";

test("release preflight validates every real-project shard artifact", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 500);
  const artifacts = realProjectArtifacts(run);

  await assert.doesNotReject(() =>
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts,
      registry: typecheckRegistry(),
      readArtifactEntries: async (artifact) => shardEntries(Number(artifact.name.split("-").pop())),
    }),
  );
});

test("release preflight aggregates typecheck evidence across non-typecheck shards", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 505);
  const artifacts = realProjectArtifacts(run);

  await assert.doesNotReject(() =>
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts,
      registry: typecheckRegistry(["fixture-0", "fixture-9"]),
      readArtifactEntries: async (artifact) => {
        const shard = Number(artifact.name.split("-").pop());
        return shardEntries(shard, {
          typecheckProject: shard === 0 || shard === 9 ? `fixture-${shard}` : null,
        });
      },
    }),
  );
});

test("release preflight rejects incomplete or duplicate aggregate typecheck coverage", async () => {
  for (const [label, registry, projectForShard, message] of [
    [
      "missing",
      typecheckRegistry(["fixture-0", "fixture-9"]),
      (shard: number) => (shard === 0 ? "fixture-0" : null),
      /missing typecheck performance projects: fixture-9/,
    ],
    [
      "duplicate",
      typecheckRegistry(["fixture-0"]),
      (shard: number) => (shard === 0 || shard === 9 ? "fixture-0" : null),
      /duplicates typecheck performance release evidence for fixture-0/,
    ],
    [
      "unexpected",
      typecheckRegistry(["fixture-0"]),
      (shard: number) => (shard === 0 ? "fixture-0" : shard === 9 ? "fixture-9" : null),
      /unregistered typecheck performance project fixture-9/,
    ],
  ] as const) {
    const run = successfulReleaseRun("Real Project Matrix", 506);
    await assert.rejects(
      assertRealProjectMatrixReleaseArtifacts({
        run,
        artifacts: realProjectArtifacts(run),
        registry,
        readArtifactEntries: async (artifact) => {
          const shard = Number(artifact.name.split("-").pop());
          return shardEntries(shard, { typecheckProject: projectForShard(shard) });
        },
      }),
      message,
      label,
    );
  }
});

test("release preflight rejects missing or foreign real-project shard artifacts", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 501);
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run).filter(
        (artifact) => artifact.name !== "real-project-matrix-7",
      ),
      registry: typecheckRegistry(),
      readArtifactEntries: async (artifact) => shardEntries(Number(artifact.name.split("-").pop())),
    }),
    /real-project-matrix-7 artifact; found 0/,
  );
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run).map((artifact) =>
        artifact.name === "real-project-matrix-0"
          ? { ...artifact, workflow_run: { ...artifact.workflow_run, head_sha: "b".repeat(40) } }
          : artifact,
      ),
      registry: typecheckRegistry(),
      readArtifactEntries: async () => shardEntries(0),
    }),
    /not bound to run/,
  );
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run).map((artifact) =>
        artifact.name === "real-project-matrix-0"
          ? { ...artifact, workflow_run: undefined }
          : artifact,
      ),
      registry: typecheckRegistry(),
      readArtifactEntries: async () => shardEntries(0),
    }),
    /not bound to run/,
  );
});

// `enforceParity: true` is passed explicitly because the shipped default is
// waived while #4461 is open. These cases keep proving the strict path works,
// so flipping `releaseTypecheckParityEnforced` back to `true` needs no new test.
test("release preflight rejects record-only and non-zero typecheck parity artifacts", async () => {
  for (const [label, mutate, message] of [
    [
      "record-only",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => (artifact.enforcement.budgetMode = "record-only")),
      /must not be record-only/,
    ],
    [
      "breached budget",
      (entries: Record<string, string>) =>
        mutateDivergence(entries, (artifact) => {
          artifact.budget.passed = false;
          artifact.budget.verdict = "breached";
        }),
      /budget is breached/,
    ],
    [
      "false positive",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.divergence.summary.falsePositiveCount = 1),
        ),
      /zero unexplained false positives and false negatives/,
    ],
    [
      "false negative",
      (entries: Record<string, string>) =>
        mutateDivergence(
          entries,
          (artifact) => (artifact.divergence.summary.falseNegativeCount = 1),
        ),
      /zero unexplained false positives and false negatives/,
    ],
  ] as const) {
    const run = successfulReleaseRun("Real Project Matrix", 510);
    await assert.rejects(
      assertRealProjectMatrixReleaseArtifacts({
        run,
        artifacts: realProjectArtifacts(run),
        registry: typecheckRegistry(),
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

test("release preflight waives typecheck parity by default and still binds the evidence", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 515);
  const warnings: string[] = [];
  const restore = console.warn;
  console.warn = (message: string) => warnings.push(String(message));
  try {
    await assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run),
      registry: typecheckRegistry(),
      readArtifactEntries: async (artifact) => {
        const entries = shardEntries(Number(artifact.name.split("-").pop()));
        if (artifact.name === "real-project-matrix-0") {
          mutateDivergence(entries, (artifact) => {
            artifact.enforcement.budgetMode = "record-only";
            artifact.divergence.summary.falseNegativeCount = 204;
          });
        }
        return entries;
      },
    });
  } finally {
    console.warn = restore;
  }
  assert.equal(warnings.length, 1);
  assert.match(warnings[0], /waived typecheck parity \(#4461\)/);
  assert.match(warnings[0], /must not be record-only/);
});

test("release preflight still rejects unbound typecheck evidence while parity is waived", async () => {
  const run = successfulReleaseRun("Real Project Matrix", 517);
  await assert.rejects(
    assertRealProjectMatrixReleaseArtifacts({
      run,
      artifacts: realProjectArtifacts(run),
      registry: typecheckRegistry(),
      readArtifactEntries: async (artifact) => {
        const entries = shardEntries(Number(artifact.name.split("-").pop()));
        if (artifact.name === "real-project-matrix-0") {
          mutateDivergence(entries, (artifact) => {
            artifact.evidence.commitSha = "c".repeat(40);
          });
        }
        return entries;
      },
    }),
    /not bound to/,
  );
});

test("release preflight requires exact lint divergence evidence", async () => {
  for (const [label, mutate, message] of [
    [
      "missing",
      (entries: Record<string, string>) => delete entries["lint-divergence-summary.json"],
      /lint-divergence-summary\.json/,
    ],
    [
      "foreign commit",
      (entries: Record<string, string>) => {
        const summary = JSON.parse(entries["lint-divergence-summary.json"]);
        summary.evidence.commitSha = "b".repeat(40);
        entries["lint-divergence-summary.json"] = `${JSON.stringify(summary, null, 2)}\n`;
      },
      /lint divergence summary is not exact release evidence/,
    ],
  ] as const) {
    const run = successfulReleaseRun("Real Project Matrix", 520);
    await assert.rejects(
      assertRealProjectMatrixReleaseArtifacts({
        run,
        artifacts: realProjectArtifacts(run),
        registry: typecheckRegistry(),
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

test("release preflight requires a selected Real Project Matrix run", () => {
  const run = successfulReleaseRun("Real Project Matrix", 530);
  assert.equal(requireRealProjectMatrixRun(new Map([["Real Project Matrix", run]])), run);
  for (const selected of [new Map(), new Map([["Benchmark", run]])] as Map<string, unknown>[]) {
    assert.throws(
      () => requireRealProjectMatrixRun(selected),
      /Real Project Matrix release evidence is required/,
    );
  }
});
