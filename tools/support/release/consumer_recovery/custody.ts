import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { lstatSync, mkdtempSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { verifyConsumer } from "./consumer.ts";
import {
  admitOriginal,
  assertFresh,
  compareText,
  consumerAttempt,
  fixture,
  object,
  positive,
  repository,
  recoveryRunIdentity,
  reviewedPolicy,
  roles,
  same,
  sha256,
  workflow,
  type Admission,
  type Identity,
  type Json,
} from "./policy.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
export function command(executable: string, args: string[], cwd = root): Buffer {
  return execFileSync(executable, args, {
    cwd,
    timeout: 120_000,
    maxBuffer: 32 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
}
export const gitBlob = (sha: string, file: string) =>
  command("git", ["--no-replace-objects", "show", `${sha}:${file}`]).toString();
export const api = (endpoint: string): Json =>
  object(JSON.parse(command("gh", ["api", `repos/${repository}/${endpoint}`]).toString()));
export async function pages(endpoint: string, key: string): Promise<Json[]> {
  const rows: Json[] = [];
  let total = 0;
  for (let page = 1; page <= 10; page++) {
    const data = api(`${endpoint}${endpoint.includes("?") ? "&" : "?"}per_page=100&page=${page}`);
    if (page === 1) total = Number(data.total_count);
    same(data.total_count, total, "inventory total changed");
    assert.ok(
      Number.isSafeInteger(total) && total > 0 && total <= 1000,
      "bounded nonempty inventory required",
    );
    const batch = data[key] as unknown[];
    assert.ok(Array.isArray(batch) && batch.length <= 100);
    rows.push(...batch.map(object));
    assert.ok(rows.length <= total);
    if (rows.length === total) return rows;
    same(batch.length, 100, "partial intermediate inventory");
  }
  throw new Error("incomplete inventory");
}
export function authority(sha: string) {
  const directory = fileURLToPath(new URL("./", import.meta.url));
  const files = readdirSync(directory)
    .filter((file) => file.endsWith(".ts"))
    .map((file) => `tools/support/release/consumer_recovery/${file}`);
  files.push(workflow);
  return files.sort().map((file) => {
    const bytes = readFileSync(path.join(root, file));
    same(sha256(gitBlob(sha, file)), sha256(bytes), "policy checkout differs from reviewed source");
    return { file, sha256: sha256(bytes) };
  });
}
export function policyOnMain(sha: string) {
  assert.match(sha, /^[0-9a-f]{40}$/);
  const compare = api(`compare/${sha}...main`);
  const commit = api(`commits/${sha}`);
  reviewedPolicy(sha, compare, commit);
  return authority(sha);
}
export interface Original extends Json {
  schema: string;
  identity: Identity;
  policy: Json;
  run: Json;
  jobs: Json[];
  failed: Json;
  log: string;
  blobs: Json;
  artifacts: Json[];
}
export function admission(original: Original): Admission {
  same(original.schema, "vize-consumer-recovery-original-v1", "original schema mismatch");
  return admitOriginal(
    original.identity,
    original.run,
    original.jobs,
    original.failed,
    original.log,
    original.blobs,
  );
}
export async function collectOriginal(expected: Identity, policySha: string): Promise<Original> {
  const policy = { sha: policySha, authority: policyOnMain(policySha), ...fixture };
  const run = api(`actions/runs/${expected.run}`);
  const jobs = await pages(`actions/runs/${expected.run}/jobs?filter=latest`, "jobs");
  const failures = jobs.filter((job) => job.conclusion === "failure");
  same(failures.length, 1, "one known failed consumer required");
  const failed = api(`actions/jobs/${positive(failures[0].id)}`);
  const log = command("gh", [
    "api",
    `repos/${repository}/actions/jobs/${positive(failed.id)}/logs`,
  ]).toString();
  assert.ok(
    Buffer.byteLength(log) > 0 && Buffer.byteLength(log) <= 16 * 1024 * 1024,
    "bounded complete failed log required",
  );
  const blobs = {
    releaseWorkflowSha256: gitBlob(expected.head, ".github/workflows/release.yml"),
    workflowSha256: gitBlob(expected.head, ".github/workflows/release-jsr.yml"),
    consumerSha256: gitBlob(expected.head, "tools/support/release/jsr/consumer.ts"),
    denoConfigSha256: gitBlob(policySha, "tools/support/release/jsr/consumer-deno.json"),
  };
  const artifacts = await pages(`actions/runs/${expected.run}/artifacts`, "artifacts");
  for (const artifact of artifacts) {
    positive(artifact.id);
    assert.ok(
      typeof artifact.name === "string" && artifact.expired === false,
      "original artifact expired/ambiguous",
    );
    assert.match(
      String(artifact.digest),
      /^sha256:[0-9a-f]{64}$/,
      "original artifact digest required",
    );
  }
  const original: Original = {
    schema: "vize-consumer-recovery-original-v1",
    identity: expected,
    policy,
    run,
    jobs,
    failed,
    log,
    blobs,
    artifacts,
  };
  admission(original);
  same(api(`actions/runs/${expected.run}`), run, "R changed during original collection");
  return original;
}
export function verifyIntegration(github: Json, expected: Identity) {
  const lines = String(github.annotation).split(/\r?\n/);
  const field = (key: string) => {
    const values = lines
      .filter((line) => line.startsWith(`${key}: `))
      .map((line) => line.slice(key.length + 2));
    same(values.length, 1, `unique annotated ${key} required`);
    return values[0];
  };
  same(field("Validated-run"), expected.run, "tag R mismatch");
  same(field("Validated-head"), expected.head, "tag H mismatch");
  same(field("Validated-base"), expected.cut, "tag C mismatch");
  same(
    field("Protected-candidate"),
    field("Integration-merge"),
    "original protected candidate must be actual signed delivery",
  );
  same(
    field("Integration-PR"),
    String(github.integrationPr),
    "source/tag integration receipt mismatch",
  );
  const integration = api(`pulls/${positive(field("Integration-PR"))}`);
  same(integration.merged, true, "original integration must be merged");
  same(object(integration.head).sha, field("Integration-head"), "integration head mismatch");
  same(integration.merge_commit_sha, field("Integration-merge"), "integration merge mismatch");
  same(object(integration.base).ref, "main", "integration base mismatch");
  for (const side of ["head", "base"])
    same(object(object(integration[side]).repo).full_name, repository, "foreign integration");
  const body = String(integration.body);
  for (const [key, value] of [
    ["vize-release-pin-source", expected.sourcePr],
    ["vize-release-pin-head", expected.head],
    ["vize-release-pin-cut", expected.cut],
    ["vize-release-pin-tag", expected.tag],
  ]) {
    same(
      body.split(`<!-- ${key}: ${value} -->`).length,
      2,
      `original integration marker ${key} required`,
    );
    same(
      (body.match(new RegExp(`<!--\\s*${key}\\s*:`, "g")) ?? []).length,
      1,
      `ambiguous original integration marker ${key}`,
    );
  }
  const merge = api(`commits/${field("Integration-merge")}`);
  const verification = object(object(merge.commit).verification);
  same(verification.verified, true, "original signed merge required");
  same(verification.reason, "valid", "original merge signature invalid");
  same(
    (merge.parents as unknown[]).map((parent) => object(parent).sha),
    [field("Integration-parent")],
    "original integration parent mismatch",
  );
  const gates = api(
    `actions/workflows/check.yml/runs?head_sha=${field("Protected-candidate")}&per_page=100`,
  );
  assert.ok(
    (gates.workflow_runs as unknown[])
      .map(object)
      .some(
        (run) =>
          run.head_sha === field("Protected-candidate") &&
          run.event === "push" &&
          run.head_branch === "main" &&
          run.status === "completed" &&
          run.conclusion === "success",
      ),
    "successful exact original delivery Check required",
  );
  return { integration, merge, gates };
}
export function files(directory: string) {
  const found: { file: string; sha256: string; bytes: number }[] = [];
  let total = 0;
  function visit(relative: string) {
    for (const name of readdirSync(path.join(directory, relative)).sort()) {
      const file = path.posix.join(relative, name);
      const absolute = path.join(directory, file);
      const info = lstatSync(absolute);
      assert.ok(!info.isSymbolicLink(), "recovery artifact symlinks forbidden");
      if (info.isDirectory()) {
        visit(file);
        continue;
      }
      assert.ok(info.isFile(), "normal evidence file required");
      if (["receipt.json", "receipt.sigstore.json"].includes(file)) continue;
      total += info.size;
      assert.ok(
        total <= 64 * 1024 * 1024 && found.length < 500,
        "bounded complete recovery evidence required",
      );
      found.push({ file, sha256: sha256(readFileSync(absolute)), bytes: info.size });
    }
  }
  visit("");
  return found;
}
export async function verifiedRecovery(
  expected: Identity,
  recoveryRun: string,
): Promise<{ admission: Admission; receipt: Json; directory: string }> {
  positive(recoveryRun);
  const run = api(`actions/runs/${recoveryRun}`);
  recoveryRunIdentity(run, recoveryRun);
  const attempt = positive(run.run_attempt);
  const policySha = String(run.head_sha);
  const policyAuthority = policyOnMain(policySha);
  const jobs = await pages(`actions/runs/${recoveryRun}/jobs?filter=latest`, "jobs");
  same(jobs.length, 9, "admission contracts, prepare, six consumers and signed finalizer required");
  same(
    jobs.map((job) => job.name).sort(compareText),
    [
      "Admit frozen consumer failure",
      "Consumer recovery admission contracts",
      "Sign consumer replacement receipt",
      ...roles.map((role) => `Replay ${role}`),
    ].sort(),
    "exact fresh consumer workflow jobs required",
  );
  assert.ok(
    jobs.every(
      (job) =>
        job.status === "completed" &&
        job.conclusion === "success" &&
        String(job.run_id) === recoveryRun &&
        job.head_sha === policySha &&
        job.run_attempt === attempt,
    ),
    "all fresh recovery jobs must pass in same attempt",
  );
  const artifacts = await pages(`actions/runs/${recoveryRun}/artifacts`, "artifacts");
  const name = `consumer-recovery-${recoveryRun}-${attempt}`;
  const matches = artifacts.filter(
    (artifact) => artifact.name === name && artifact.expired === false,
  );
  same(matches.length, 1, "one complete signed replacement artifact required");
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-verified-consumer-recovery-"));
  command("gh", [
    "run",
    "download",
    recoveryRun,
    "--repo",
    repository,
    "--name",
    name,
    "--dir",
    directory,
  ]);
  const receiptFile = path.join(directory, "receipt.json");
  const receipt = object(JSON.parse(readFileSync(receiptFile, "utf8")));
  command("cosign", [
    "verify-blob",
    "--bundle",
    path.join(directory, "receipt.sigstore.json"),
    "--certificate-identity",
    `https://github.com/${repository}/${workflow}@refs/heads/main`,
    "--certificate-oidc-issuer",
    "https://token.actions.githubusercontent.com",
    "--certificate-github-workflow-sha",
    policySha,
    "--certificate-github-workflow-repository",
    repository,
    "--certificate-github-workflow-ref",
    "refs/heads/main",
    "--certificate-github-workflow-trigger",
    "workflow_dispatch",
    receiptFile,
  ]);
  assertFresh(receipt, expected);
  same(
    receipt.recovery,
    { run: recoveryRun, attempt, policySha },
    "replacement run/attempt/source mismatch",
  );
  same(receipt.authority, policyAuthority, "replacement policy changed");
  same(receipt.files, files(directory), "complete raw replacement observations changed");
  const original = JSON.parse(
    readFileSync(path.join(directory, "original.json"), "utf8"),
  ) as Original;
  same(original.identity, expected, "original identity reused");
  const admitted = admission(original);
  for (const role of roles) {
    const packet = object(
      JSON.parse(readFileSync(path.join(directory, "consumers", role, "consumer.json"), "utf8")),
    );
    verifyConsumer(packet, original);
    same(packet.role, role, "replacement role substituted");
    consumerAttempt(packet.run, recoveryRun, attempt, policySha);
  }
  const live = await collectOriginal(expected, policySha);
  for (const key of ["policy", "run", "jobs", "failed", "log", "blobs", "artifacts"])
    same(original[key], live[key], `original ${key} changed after requalification`);
  same(api(`actions/runs/${recoveryRun}`), run, "replacement attempt changed during verification");
  assert.ok(realpathSync(directory) === directory, "replacement root redirected");
  return { admission: admitted, receipt, directory };
}
