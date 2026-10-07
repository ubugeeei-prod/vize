import assert from "node:assert/strict";
import { test } from "node:test";

import { selectCurrentMatrixArtifacts } from "../../../tools/support/compat/github/release-preflight-matrix-selection.mjs";
import { assertRealProjectMatrixReleaseArtifacts } from "../../../tools/support/compat/github/release-preflight-matrix-evidence.mjs";
import {
  shardEntries,
  typecheckRegistry,
} from "../_helpers/release-preflight-matrix-evidence-fixture.ts";
import { retriedMatrixEvidence } from "../support/release-matrix-attempt-fixture.ts";

function select(fixture: ReturnType<typeof retriedMatrixEvidence>) {
  return selectCurrentMatrixArtifacts({
    ...fixture,
    readPreviousJobs: async (attempt: number) => {
      assert.equal(attempt, 1);
      return fixture.previousJobs;
    },
  });
}

test("failed-only retry selects the successful lower-ID artifact and original carried jobs", async () => {
  const fixture = retriedMatrixEvidence();
  const before = structuredClone(fixture);
  const selected = await select(fixture);
  assert.equal(selected.artifacts.length, 22);
  assert.equal(selected.artifacts[19].id, 3_019);
  assert.deepEqual(selected.provenance[19].historicalArtifacts, [
    fixture.artifacts[19],
    fixture.artifacts[22],
  ]);
  assert.equal(selected.provenance[19].producingJob.id, 2_019);
  assert.equal(selected.provenance[19].carriedForward, false);
  assert.equal(selected.provenance[0].producingJob.id, 1_000);
  assert.equal(selected.provenance[0].producingJob.run_attempt, 1);
  assert.equal(selected.provenance[0].observedJob.id, 2_000);
  assert.equal(selected.provenance[0].carriedForward, true);
  assert.deepEqual(fixture, before, "selection preserves all historical metadata");
});

test("selected archives still require complete reports and the actual successful surface verdict", async () => {
  const fixture = retriedMatrixEvidence();
  const selected = await select(fixture);
  const readIds: number[] = [];
  const validate = (omitVerdict: boolean) =>
    assertRealProjectMatrixReleaseArtifacts({
      run: fixture.run,
      artifacts: selected.artifacts,
      registry: typecheckRegistry(),
      readArtifactEntries: async (artifact: { id: number; name: string }) => {
        readIds.push(artifact.id);
        assert.notEqual(artifact.id, 9_999, "partial failed archive must remain historical");
        const shard = Number(artifact.name.split("-").pop());
        const entries = shardEntries(shard);
        if (omitVerdict && shard === 19) delete entries["surface-verdict.json"];
        return entries;
      },
    });
  await assert.doesNotReject(() => validate(false));
  assert.deepEqual(
    readIds,
    selected.artifacts.map((artifact: { id: number }) => artifact.id),
  );
  await assert.rejects(() => validate(true), /surface-verdict\.json/);
});

test("selection refuses ambiguous or absent evidence from the current successful execution", async () => {
  const duplicate = retriedMatrixEvidence();
  duplicate.artifacts.push({ ...duplicate.artifacts[22], id: 3_020 });
  await assert.rejects(() => select(duplicate), /successful current execution; found 2/);
  const missing = retriedMatrixEvidence();
  missing.artifacts.pop();
  await assert.rejects(() => select(missing), /successful current execution; found 0/);
  const unknown = retriedMatrixEvidence();
  unknown.artifacts[22].created_at = "2026-10-08T00:05:00Z";
  await assert.rejects(() => select(unknown), /unknown or ambiguous producing execution/);
  const expired = retriedMatrixEvidence();
  expired.artifacts[22].expired = true;
  await assert.rejects(() => select(expired), /expired/);
});

test("selection rejects foreign source, run, branch and attempt metadata", async () => {
  for (const field of ["head_sha", "head_branch", "id"] as const) {
    const fixture = retriedMatrixEvidence();
    Object.assign(fixture.artifacts[22].workflow_run, {
      [field]: field === "id" ? 901 : "b".repeat(40),
    });
    await assert.rejects(() => select(fixture), /artifact is not bound/);
  }
  for (const field of ["head_sha", "run_id", "run_attempt"] as const) {
    const fixture = retriedMatrixEvidence();
    Object.assign(fixture.currentJobs[19], {
      [field]: field === "head_sha" ? "b".repeat(40) : 999,
    });
    await assert.rejects(() => select(fixture), /job is not bound/);
  }
});

test("copied-forward aliases require the original authenticated execution snapshot", async () => {
  const missingHistory = retriedMatrixEvidence();
  missingHistory.previousJobs = [];
  await assert.rejects(() => select(missingHistory), /carried-forward execution/);
  const duplicateHistory = retriedMatrixEvidence();
  duplicateHistory.previousJobs.push({ ...duplicateHistory.previousJobs[0], id: 4_000 });
  await assert.rejects(() => select(duplicateHistory), /ambiguous historical job identities/);
  const duplicateCurrent = retriedMatrixEvidence();
  duplicateCurrent.currentJobs.push({ ...duplicateCurrent.currentJobs[0], id: 4_000 });
  await assert.rejects(() => select(duplicateCurrent), /exactly one current job; found 2/);
});

test("selection fails closed on missing stamps, failed jobs and invalid execution times", async () => {
  const missingAttempt = retriedMatrixEvidence();
  Reflect.deleteProperty(missingAttempt.run, "run_attempt");
  await assert.rejects(() => select(missingAttempt), /positive run_attempt/);
  const failed = retriedMatrixEvidence();
  failed.currentJobs[19].conclusion = "failure";
  await assert.rejects(() => select(failed), /current producing job is not successful/);
  for (const value of ["2026-10-08T00:10:60Z", "2026-02-30T00:00:00Z", "invalid"]) {
    const fixture = retriedMatrixEvidence();
    fixture.currentJobs[19].started_at = value;
    await assert.rejects(() => select(fixture), /invalid started_at timestamp/);
  }
  const outside = retriedMatrixEvidence();
  outside.currentJobs[19].completed_at = "2026-10-08T00:21:00Z";
  await assert.rejects(() => select(outside), /inconsistent execution timestamps/);
});
