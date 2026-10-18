/** Whole artifact, immutable source/main and durable first-publication custody. */
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  artifactName,
  assertReceipt,
  candidate,
  positive,
  record,
  workflow,
  type Candidate,
} from "./policy.ts";
import { sha256 } from "./registry.ts";
import type { Commands } from "./main.ts";
type Qualify = (row: unknown, value: Candidate, io: Commands) => number;

export const checkpointPath = "tools/support/release/corsa_bootstrap/completed.json";
// A future ordinary PR may retain one actual complete handoff after publication.
// No fabricated placeholder or caller-supplied admission flag is accepted.
export const policyFiles = [
  workflow,
  ...["policy", "registry", "main", "carrier", "custody"].map(
    (name) => `tools/support/release/corsa_bootstrap/${name}.ts`,
  ),
  ...["main", "manifest", "registry", "config"].map(
    (name) => `tools/support/release/publish_crates/${name}.ts`,
  ),
  ...["command_io", "command_runtime", "ordered_json"].map(
    (name) => `tools/support/release/${name}.ts`,
  ),
  ...["main", "check-state", "overlap"].map(
    (name) => `tools/support/release/qualify_check/${name}.ts`,
  ),
  ...["bootstrap", "evidence", "github"].map(
    (name) => `tools/support/compat/github/release-preflight-${name}.mjs`,
  ),
  "tools/support/compat/github/real-project-matrix-plan.ts",
  ...["public-consumer", "package-consumer", "contract", "source-consumers"].map(
    (name) => `tools/support/compat/github/checked-corsa-${name}.ts`,
  ),
  "tests/_fixtures/ci/checked-corsa-consumer/Cargo.toml",
  "tests/_fixtures/ci/checked-corsa-consumer/src/lib.rs",
] as const;

function assertCarrier(row: unknown, value: Candidate, id: number, attempt: number, login: string) {
  const run = record(row);
  assert.equal(run.id, id);
  assert.equal(run.run_attempt, attempt);
  assert.equal(run.path, workflow);
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.head_sha, value.head);
  assert.equal(run.head_branch, value.branch);
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  assert.equal(record(run.repository).full_name, value.repository);
  assert.equal(record(run.actor).login, login);
  assert.equal(record(run.triggering_actor).login, login);
}

/** A reviewed committed original receipt survives Actions artifact expiry. */
export function readCheckpoint(value: Candidate, io: Commands, qualify: Qualify) {
  if (!io.exists(checkpointPath)) return undefined;
  const checkpoint = record(JSON.parse(io.read(checkpointPath)));
  assert.equal(checkpoint.schema, "vize-corsa-completed-checkpoint-v1");
  const stored = record(checkpoint.receipt);
  const original = candidate(record(stored.candidate) as Candidate);
  positive(stored.run, "original carrier run");
  positive(stored.attempt, "original carrier attempt");
  const receipt = assertReceipt(stored, original, stored.run, stored.attempt);
  const main = String(checkpoint.mainCommit);
  assert.match(main, /^[0-9a-f]{40}$/u);
  const commit = record(io.api(`repos/${value.repository}/commits/${main}`));
  assert.equal(commit.sha, main);
  assert.equal(record(record(commit.commit).verification).verified, true);
  positive(checkpoint.releaseRun, "successful original official Release");
  const release = record(io.api(`repos/${value.repository}/actions/runs/${checkpoint.releaseRun}`));
  assert.equal(release.path, ".github/workflows/release.yml");
  assert.equal(release.event, "workflow_dispatch");
  assert.equal(release.head_sha, original.head);
  assert.equal(release.head_branch, original.branch);
  assert.equal(release.status, "completed");
  assert.equal(release.conclusion, "success");
  assert.equal(record(release.repository).full_name, original.repository);
  const title = `Release ${original.tag} PR #${original.number} @ ${original.head}`;
  assert.ok(release.display_title === title || release.display_title === `Pinned ${title}`);
  io.run("git", ["fetch", "--no-tags", "--no-write-fetch-head", "origin", original.head, main]);
  assert.equal(
    io.git("rev-parse", `${original.head}^{tree}`),
    io.git("rev-parse", `${main}^{tree}`),
    "original H/main whole tree differs",
  );
  io.git("merge-base", "--is-ancestor", main, value.cut);
  const hashes = record(checkpoint.policy);
  assert.deepEqual(
    Object.keys(hashes).sort(),
    [...policyFiles].sort(),
    "closed original policy table required",
  );
  for (const file of policyFiles) {
    assert.match(String(hashes[file]), /^[0-9a-f]{64}$/u);
    assert.equal(
      sha256(io.run("git", ["--no-replace-objects", "show", `${original.head}:${file}`])),
      hashes[file],
    );
    assert.equal(
      sha256(io.read(file)),
      hashes[file],
      "changed carrier policy requires explicit checkpoint review",
    );
  }
  assertCarrier(
    io.api(`repos/${value.repository}/actions/runs/${receipt.run}`),
    original,
    receipt.run,
    receipt.attempt,
    receipt.ownerLogin,
  );
  const jobs = record(
    io.api(
      `repos/${value.repository}/actions/runs/${receipt.run}/attempts/${receipt.attempt}/jobs?per_page=100`,
    ),
  );
  assert.ok(Array.isArray(jobs.jobs) && Number(jobs.total_count) <= 100);
  const bootstrap = jobs.jobs.map(record).filter((job) => job.name === "bootstrap");
  assert.equal(bootstrap.length, 1);
  assert.equal(bootstrap[0].conclusion, "success", "actual protected carrier job must have run");
  qualify(io.api(`repos/${value.repository}/actions/runs/${receipt.qualification}`), original, io);
  assert.equal(
    record(record(io.api(`repos/${value.repository}`)).owner).id,
    receipt.repositoryOwner,
  );
  assertCarrier(
    io.api(`repos/${value.repository}/actions/runs/${receipt.run}`),
    original,
    receipt.run,
    receipt.attempt,
    receipt.ownerLogin,
  );
  return receipt;
}

export function readHandoff(value: Candidate, io: Commands, qualify: Qualify) {
  const response = record(
    io.api(
      `repos/${value.repository}/actions/workflows/release-corsa-bootstrap.yml/runs?event=workflow_dispatch&head_sha=${value.head}&per_page=100`,
    ),
  );
  assert.ok(Array.isArray(response.workflow_runs));
  assert.ok(Number(response.total_count) <= 100, "ambiguous truncated bootstrap inventory");
  const successful = response.workflow_runs
    .map(record)
    .filter((run) => run.conclusion === "success");
  assert.ok(successful.length <= 1, "multiple successful bootstrap carriers require inspection");
  if (successful.length === 0) return undefined;
  const run = structuredClone(successful[0]);
  positive(run.id, "bootstrap run");
  positive(run.run_attempt, "bootstrap attempt");
  assert.equal(run.path, workflow);
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.head_sha, value.head);
  assert.equal(run.head_branch, value.branch);
  assert.equal(run.status, "completed");
  assert.equal(record(run.repository).full_name, value.repository);
  const artifacts = record(
    io.api(`repos/${value.repository}/actions/runs/${run.id}/artifacts?per_page=100`),
  );
  assert.ok(Array.isArray(artifacts.artifacts));
  assert.ok(Number(artifacts.total_count) <= 100);
  const matches = artifacts.artifacts.map(record).filter((row) => row.name === artifactName);
  assert.equal(matches.length, 1, "exactly one bootstrap custody artifact required");
  const artifact = matches[0];
  positive(artifact.id, "artifact ID");
  assert.equal(artifact.expired, false);
  assert.match(String(artifact.digest), /^sha256:[0-9a-f]{64}$/u);
  assert.equal(record(artifact.workflow_run).id, run.id);
  assert.equal(record(artifact.workflow_run).head_sha, value.head);
  const zip = io.run("gh", [
    "api",
    `repos/${value.repository}/actions/artifacts/${artifact.id}/zip`,
  ]);
  assert.equal(`sha256:${sha256(zip)}`, artifact.digest, "whole artifact ZIP digest mismatch");
  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-corsa-handoff-"));
  try {
    const archive = path.join(temporary, "handoff.zip");
    writeFileSync(archive, zip);
    assert.equal(
      io.run("unzip", ["-Z1", archive]).toString(),
      "receipt.json\n",
      "bounded single-member artifact required",
    );
    const receipt = assertReceipt(
      JSON.parse(io.run("unzip", ["-p", archive, "receipt.json"]).toString()),
      value,
      run.id,
      run.run_attempt,
    );
    assertCarrier(run, value, receipt.run, receipt.attempt, receipt.ownerLogin);
    qualify(io.api(`repos/${value.repository}/actions/runs/${receipt.qualification}`), value, io);
    const repository = record(io.api(`repos/${value.repository}`));
    assert.equal(record(repository.owner).id, receipt.repositoryOwner);
    assertCarrier(
      io.api(`repos/${value.repository}/actions/runs/${receipt.run}`),
      value,
      receipt.run,
      receipt.attempt,
      receipt.ownerLogin,
    );
    return receipt;
  } finally {
    rmSync(temporary, { recursive: true });
  }
}
