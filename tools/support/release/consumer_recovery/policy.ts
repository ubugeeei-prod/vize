import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { stripVTControlCharacters } from "node:util";

export const repository = "ubugeeei-prod/vize";
export const workflow = ".github/workflows/release-consumer-recovery.yml";
export const roles = [
  "node22-linux",
  "node24-linux",
  "node24-macos",
  "node24-windows",
  "deno-linux",
  "bun-linux",
] as const;
export const fixture = {
  releaseWorkflowSha256: "d3b8183edd0cddd579b7f88ddc01417ab51890ee9577e3a197dd59d654a1b5fa",
  workflowSha256: "bf5b8d553c05b85afa434a4b4f637a5c5f0b09f9357a6026ab4d0f2014893d81",
  consumerSha256: "410be5021e5c1bff706191bf68045ddbc2edbaa47b13325ddde3daa22926851c",
  denoConfigSha256: "7b02ff3799e9e65083d27c1f65260c9a294bb809e79174f828f9bd79148b9700",
  // This exact graph includes 54 successful jobs, the one conditional handoff
  // skip and the defective Deno consumer. Missing or newly skipped gates refuse.
  jobsSha256: "51321b34836eeb5a9ea9342452ce5280e35538c352a0c80e0e26b7ee512dabea",
  failedJob: "Release @vizejs/vize to JSR / JSR consumer (deno, ubuntu-24.04)",
};
export type Json = Record<string, unknown>;
export function object(value: unknown): Json {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "object required");
  return value as Json;
}
export const sha256 = (value: string | Uint8Array) =>
  createHash("sha256").update(value).digest("hex");
export const compareText = (left: unknown, right: unknown) =>
  String(left) < String(right) ? -1 : String(left) > String(right) ? 1 : 0;
export function same(actual: unknown, expected: unknown, message: string) {
  assert.deepEqual(actual, expected, message);
}
export function positive(value: unknown): number {
  assert.ok(
    Number.isSafeInteger(Number(value)) && /^[1-9][0-9]*$/.test(String(value)),
    "positive exact identity required",
  );
  return Number(value);
}
export function failureLogArguments(id: unknown) {
  return [
    "api",
    "--allow-escape-sequences",
    `repos/${repository}/actions/jobs/${positive(id)}/logs`,
  ];
}
export interface Identity {
  cut: string;
  head: string;
  tag: string;
  sourcePr: string;
  run: string;
  attempt: number;
}
export function identity(environment: NodeJS.ProcessEnv): Identity {
  const result = {
    cut: environment.PUBLIC_CUT ?? "",
    head: environment.PUBLIC_HEAD ?? "",
    tag: environment.PUBLIC_TAG ?? "",
    sourcePr: environment.PUBLIC_SOURCE_PR ?? "",
    run: environment.PUBLIC_RELEASE_RUN ?? "",
    attempt: positive(environment.PUBLIC_RELEASE_ATTEMPT),
  };
  for (const value of [result.cut, result.head]) assert.match(value, /^[0-9a-f]{40}$/);
  assert.match(result.tag, /^v0\.[1-9][0-9]*\.0$/);
  positive(result.sourcePr);
  positive(result.run);
  return result;
}
export function runIdentity(run: Json, expected: Identity) {
  same(String(run.id), expected.run, "original R mismatch");
  same(run.head_sha, expected.head, "original H mismatch");
  same(run.head_branch, `release/${expected.tag}`, "original branch mismatch");
  same(run.event, "workflow_dispatch", "original event mismatch");
  same(run.path, ".github/workflows/release.yml", "original workflow mismatch");
  same(
    run.display_title,
    `Pinned Release ${expected.tag} PR #${expected.sourcePr} @ ${expected.head}`,
    "original source receipt mismatch",
  );
  for (const key of ["repository", "head_repository"])
    same(object(run[key]).full_name, repository, "foreign original repository");
  same(run.status, "completed", "original R must be terminal");
  same(run.conclusion, "failure", "original R must remain failed");
  same(run.run_attempt, expected.attempt, "original attempt changed");
}
// The branded authority is produced only after complete live custody admission.
declare const admitted: unique symbol;
export interface Admission {
  identity: Identity;
  failedJob: number;
  [admitted]: true;
}
export function admitOriginal(
  expected: Identity,
  run: Json,
  jobs: Json[],
  failed: Json,
  log: string,
  blobs: Json,
): Admission {
  runIdentity(run, expected);
  for (const [name, digest] of Object.entries(fixture).filter(
    ([key]) => key.endsWith("Sha256") && key !== "jobsSha256",
  )) {
    same(
      sha256(String(blobs[name])),
      digest,
      `unreviewed original/corrected source bytes: ${name}`,
    );
  }
  same(jobs.length, 56, "complete original job graph required");
  same(
    sha256(JSON.stringify(jobs.map((job) => [job.name, job.conclusion]).sort(compareText))),
    fixture.jobsSha256,
    "original gate outcomes changed",
  );
  const ids = new Set<number>();
  for (const job of jobs) {
    const id = positive(job.id);
    assert.ok(!ids.has(id), "duplicate original job");
    ids.add(id);
    same(String(job.run_id), expected.run, "foreign job R");
    same(job.head_sha, expected.head, "foreign job H");
    same(job.status, "completed", "pending original job");
    assert.ok(positive(job.run_attempt) <= expected.attempt, "future job attempt");
  }
  const failures = jobs.filter((job) => job.conclusion === "failure");
  same(failures.length, 1, "only known consumer failure may be requalified");
  same(failures[0].name, fixture.failedJob, "other failed original job");
  same(failed.id, failures[0].id, "failed job detail mismatch");
  same(failed.name, fixture.failedJob, "failed job detail name mismatch");
  same(failed.status, "completed", "failed detail pending");
  same(failed.conclusion, "failure", "failed detail conclusion changed");
  same(failed.run_attempt, expected.attempt, "failure must belong to exact latest attempt");
  same(failed.head_sha, expected.head, "failed detail H mismatch");
  same(String(failed.run_id), expected.run, "failed detail R mismatch");
  const steps = (failed.steps as unknown[]).map(object);
  same(
    steps.filter((step) => step.conclusion === "failure").map((step) => step.name),
    ["Import the published entry points (Deno)"],
    "only frozen consumer invocation may fail",
  );
  assert.ok(
    steps.every(
      (step) =>
        step.status === "completed" &&
        ["success", "skipped", "failure"].includes(String(step.conclusion)),
    ),
    "other step failure",
  );
  const plain = stripVTControlCharacters(log);
  const fingerprint = `Could not find a matching package for 'npm:vize@${expected.tag.slice(1)}' in the node_modules directory.`;
  same(
    plain.split(fingerprint).length,
    2,
    "known Deno nodeModulesDir failure fingerprint required exactly once",
  );
  assert.ok(
    plain.includes('"nodeModulesDir": "auto"') &&
      plain.includes("deno run --allow-all consumer.ts") &&
      plain.includes("Going to install stable version 2.9.7."),
    "complete original Deno command/runtime evidence required",
  );
  same((plain.match(/##\[error\]/g) ?? []).length, 1, "other logged failure");
  assert.ok(
    plain.includes("##[error]Process completed with exit code 1."),
    "complete failed command outcome required",
  );
  return { identity: expected, failedJob: positive(failed.id) } as Admission;
}
export function assertFresh(receipt: Json, expected: Identity, now = Date.now()) {
  same(receipt.schema, "vize-consumer-recovery-v1", "unknown recovery schema");
  same(receipt.identity, expected, "reused/foreign recovery identity");
  const age = now - Date.parse(String(receipt.completedAt));
  assert.ok(
    Number.isFinite(age) && age >= 0 && age <= 24 * 3600_000,
    "recovery receipt expired or future dated",
  );
  same(receipt.originalConclusion, "failure", "original failure must remain visible");
  same(receipt.status, "consumer-requalified", "incomplete recovery");
  const consumers = (receipt.consumers as unknown[]).map(object);
  same(
    consumers.map((consumer) => consumer.role).sort(compareText),
    [...roles].sort(),
    "six distinct original consumer roles required",
  );
}

export function recoveryRunIdentity(run: Json, expectedRun: string) {
  same(String(run.id), expectedRun, "recovery run identity mismatch");
  for (const key of ["repository", "head_repository"])
    same(object(run[key]).full_name, repository, "foreign recovery repository");
  same(run.event, "workflow_dispatch", "recovery dispatch required");
  same(run.path, workflow, "foreign recovery workflow");
  same(run.head_branch, "main", "reviewed main recovery required");
  same(run.status, "completed", "recovery run pending");
  same(run.conclusion, "success", "recovery workflow failed");
  assert.match(String(run.head_sha), /^[0-9a-f]{40}$/, "full reviewed policy SHA required");
  positive(run.run_attempt);
}
export function consumerAttempt(run: unknown, id: string, attempt: number, sha: string) {
  same(
    run,
    { id, attempt: String(attempt), sha },
    "reused consumer run/attempt or unverified policy source",
  );
}

export function reviewedPolicy(sha: string, compare: Json, commit: Json) {
  assert.match(sha, /^[0-9a-f]{40}$/);
  assert.ok(
    ["ahead", "identical"].includes(String(compare.status)),
    "policy must be on reviewed main",
  );
  same(compare.behind_by, 0, "foreign policy source");
  same(object(compare.base_commit).sha, sha, "policy comparison base mismatch");
  same(object(compare.merge_base_commit).sha, sha, "policy is not an ancestor of current main");
  same(commit.sha, sha, "policy commit identity mismatch");
  const verification = object(object(commit.commit).verification);
  same(verification.verified, true, "policy commit signature required");
  same(verification.reason, "valid", "valid policy commit signature required");
}
