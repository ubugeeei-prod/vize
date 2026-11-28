// Re-checks every bound Content Mapper v1 obligation against the GitHub run
// that produced its receipts: the run is at the bound commit, from the named
// workflow and attempt, every named job and step succeeded, the artifact digest
// matches, and every named test case passed in the artifact's test-cases JSON.
//
// Usage: node tools/commands/release/verify-content-mapper-bindings.ts [--repo owner/name]
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  CONTENT_MAPPER_V1_BINDINGS,
  type ContentMapperV1Receipt,
} from "../../support/compat/v1/content-mapper-bindings.ts";

const repoFlag = process.argv.indexOf("--repo");
const repo = repoFlag === -1 ? "ubugeeei-prod/vize" : process.argv[repoFlag + 1];
assert.ok(repo, "--repo needs owner/name");

function gh(...args: string[]): string {
  return execFileSync("gh", args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

function api<T>(route: string): T {
  return JSON.parse(gh("api", "--paginate", "--slurp", route)) as T;
}

interface Run {
  head_sha: string;
  path: string;
  run_attempt: number;
  conclusion: string | null;
}
interface Job {
  name: string;
  conclusion: string | null;
  steps?: { name: string; conclusion: string | null }[];
}
interface Artifact {
  name: string;
  digest: string | null;
  expired: boolean;
}

function passedCases(runId: number, artifact: string): Map<string, string> {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-binding-"));
  try {
    gh("run", "download", String(runId), "-R", repo, "-n", artifact, "-D", directory);
    const outcomes = new Map<string, string>();
    for (const file of readdirSync(directory).filter((name) => name.endsWith(".json"))) {
      const report = JSON.parse(readFileSync(path.join(directory, file), "utf8")) as {
        cases?: { name: string; outcome: string }[];
      };
      for (const { name, outcome } of report.cases ?? []) outcomes.set(name, outcome);
    }
    return outcomes;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

function verifyReceipt(candidateSource: string, receipt: ContentMapperV1Receipt): string[] {
  const label = `${receipt.workflow}#${receipt.runId}/${receipt.runAttempt} ${receipt.artifact}`;
  const run = JSON.parse(gh("api", `repos/${repo}/actions/runs/${receipt.runId}`)) as Run;
  assert.equal(run.head_sha, candidateSource, `${label}: run is not at the bound commit`);
  assert.equal(run.path, receipt.workflow, `${label}: run came from another workflow`);

  const pages = api<{ jobs: Job[] }[]>(
    `repos/${repo}/actions/runs/${receipt.runId}/attempts/${receipt.runAttempt}/jobs?per_page=100`,
  );
  const job = pages.flatMap(({ jobs }) => jobs).find(({ name }) => name === receipt.job);
  assert.ok(job, `${label}: job ${receipt.job} is missing`);
  assert.equal(job.conclusion, "success", `${label}: job ${receipt.job} did not succeed`);
  for (const step of receipt.steps) {
    const observed = job.steps?.find(({ name }) => name === step);
    assert.equal(observed?.conclusion, "success", `${label}: step ${step} did not succeed`);
  }

  const artifacts = api<{ artifacts: Artifact[] }[]>(
    `repos/${repo}/actions/runs/${receipt.runId}/artifacts?per_page=100`,
  ).flatMap(({ artifacts }) => artifacts);
  const artifact = artifacts.find(({ name }) => name === receipt.artifact);
  assert.ok(artifact, `${label}: artifact is missing`);
  assert.equal(artifact.digest, receipt.digest, `${label}: artifact digest changed`);

  const notes: string[] = [];
  if (receipt.cases.length === 0) return notes;
  if (artifact.expired) {
    notes.push(`${label}: artifact expired; digest verified, cases not re-read`);
    return notes;
  }
  const outcomes = passedCases(receipt.runId, receipt.artifact);
  for (const name of receipt.cases)
    assert.equal(outcomes.get(name), "passed", `${label}: case ${name} did not pass`);
  return notes;
}

let receipts = 0;
for (const binding of CONTENT_MAPPER_V1_BINDINGS) {
  for (const receipt of binding.receipts) {
    for (const note of verifyReceipt(binding.candidateSource, receipt)) console.log(note);
    receipts += 1;
  }
  console.log(
    `Verified ${binding.obligations.length} obligations at ${binding.candidateSource.slice(0, 10)}`,
  );
}
console.log(`Verified ${receipts} receipts in ${CONTENT_MAPPER_V1_BINDINGS.length} bindings`);
