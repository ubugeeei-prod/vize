import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { lstatSync, mkdtempSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { captureCommand, strictUtf8, verifyOriginalLogDirectory } from "./runtime.ts";
import {
  custodyProbe,
  assertOracleFiles,
  sourceAuthority,
  assertCheckoutAuthority,
  type OraclePolicy,
} from "./oracles.ts";
import { revalidatePublicArchives, type InstalledArchive } from "./archive.ts";
import {
  admitOriginal,
  compareText,
  consumerAttempt,
  fixture,
  failureLogArguments,
  integrationReference,
  integrationIdentity,
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

import {
  assertLegacyOracleFiles,
  assertRecoveryReceiptPolicy,
  collectLegacyOriginal,
  legacyAdmission,
  legacyAuthority,
  recoveryEvidencePolicy,
  verifyLegacyConsumer,
  verifyConsumerDirectory,
} from "./evidence.ts";

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
export { policyFiles } from "./oracles.ts";
export function authority(sha: string) {
  const entries = sourceAuthority(sha, gitBlob);
  assertCheckoutAuthority(entries, (file) => readFileSync(path.join(root, file)));
  return entries;
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
  policy: Json & OraclePolicy & { sha: string };
  run: Json;
  jobs: Json[];
  failed: Json;
  log: string;
  logCustody?: Json;
  blobs: Json;
  artifacts: Json[];
}
export function admission(original: Original): Admission {
  same(original.schema, "vize-consumer-recovery-original-v2", "original schema mismatch");
  return admitOriginal(
    original.identity,
    original.run,
    original.jobs,
    original.failed,
    original.log,
    original.blobs,
  );
}
export async function collectOriginal(
  expected: Identity,
  policySha: string,
  logDirectory?: string,
): Promise<Original> {
  return collectOriginalSnapshot(expected, policySha, policyOnMain(policySha), logDirectory);
}
async function collectOriginalSnapshot(
  expected: Identity,
  policySha: string,
  policyAuthority: { file: string; sha256: string }[],
  logDirectory?: string,
): Promise<Original> {
  const policy = {
    sha: policySha,
    authority: policyAuthority,
    ...fixture,
    custodyProbeSha256: sha256(custodyProbe),
  };
  const run = api(`actions/runs/${expected.run}`);
  const tagRef = api(`git/ref/tags/${expected.tag}`);
  same(object(tagRef.object).type, "tag", "original annotated tag required");
  const tagObject = api(`git/tags/${String(object(tagRef.object).sha)}`);
  same(object(tagObject.object).sha, expected.head, "original tag target H mismatch");
  const source = api(`pulls/${expected.sourcePr}`);
  const sourceReceipt = {
    number: source.number,
    title: source.title,
    body: source.body,
    draft: source.draft,
    head: {
      sha: object(source.head).sha,
      ref: object(source.head).ref,
      repository: object(object(source.head).repo).full_name,
    },
    base: { ref: object(source.base).ref, repository: object(object(source.base).repo).full_name },
  };
  const jobs = await pages(`actions/runs/${expected.run}/jobs?filter=latest`, "jobs");
  const failures = jobs.filter((job) => job.conclusion === "failure");
  same(failures.length, 1, "one known failed consumer required");
  const failed = api(`actions/jobs/${positive(failures[0].id)}`);
  const logOutput =
    logDirectory ?? path.join(mkdtempSync(path.join(os.tmpdir(), "vize-original-log-")), "capture");
  const captured = await captureCommand("gh", failureLogArguments(failed.id), logOutput);
  // Whole raw stdout/stderr and process packet exist even if any following gate refuses.
  same(captured.error, undefined, "failed log transport error");
  same(captured.signal, null, "failed log transport signal");
  same(captured.status, 0, "failed log transport exit");
  const log = strictUtf8(captured.stdout);
  const logBytes = captured.stdout;
  assert.ok(
    captured.stdout.length > 0 && captured.stdout.length <= 16 * 1024 * 1024,
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
    schema: "vize-consumer-recovery-original-v2",
    identity: expected,
    policy,
    run,
    jobs,
    failed,
    log,
    // The raw process packet retains its actual transport cwd. That local path
    // is not an original Release identity and may differ between trusted hosts.
    logCustody: Object.fromEntries(
      Object.entries(captured.custody).filter(([key]) => key !== "cwd"),
    ),
    logIdentity: { sha256: sha256(logBytes), bytes: logBytes.byteLength, encoding: "utf8" },
    blobs,
    artifacts,
    pin: { tagRef, tagObject },
    sourceReceipt,
  };
  admission(original);
  same(api(`actions/runs/${expected.run}`), run, "R changed during original collection");
  return original;
}
export function verifyIntegration(github: Json, expected: Identity, read = api) {
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
  const integrationPr = integrationReference(field("Integration-PR"), github.integrationPr);
  const integration = read(`pulls/${integrationPr}`);
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
  const merge = read(`commits/${field("Integration-merge")}`);
  const verification = object(object(merge.commit).verification);
  same(verification.verified, true, "original signed merge required");
  same(verification.reason, "valid", "original merge signature invalid");
  same(
    (merge.parents as unknown[]).map((parent) => object(parent).sha),
    [field("Integration-parent")],
    "original integration parent mismatch",
  );
  const gates = read(
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
export function verifyStoredIntegration(github: Json, expected: Identity, value: unknown) {
  const snapshot = object(value);
  const identity = integrationIdentity(snapshot);
  const responses: Record<string, Json> = {
    [`pulls/${identity.integration.number}`]: object(snapshot.integration),
    [`commits/${identity.merge.sha}`]: object(snapshot.merge),
    [`actions/workflows/check.yml/runs?head_sha=${identity.merge.sha}&per_page=100`]: object(
      snapshot.gates,
    ),
  };
  return verifyIntegration(github, expected, (endpoint) => {
    assert.ok(Object.hasOwn(responses, endpoint), "stored integration endpoint identity mismatch");
    return responses[endpoint];
  });
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
export { verifyConsumerDirectory } from "./evidence.ts";
export async function verifiedRecovery(
  expected: Identity,
  recoveryRun: string,
): Promise<{
  admission: Admission;
  receipt: Json;
  directory: string;
  integration: ReturnType<typeof verifyIntegration>;
}> {
  positive(recoveryRun);
  const run = api(`actions/runs/${recoveryRun}`);
  recoveryRunIdentity(run, recoveryRun);
  const attempt = positive(run.run_attempt);
  const policySha = String(run.head_sha);
  const evidencePolicy = recoveryEvidencePolicy(run);
  // Legacy routing is bound to the actual workflow run and reviewed signed main
  // source, then to Cosign workflow SHA below before any packet is consumed.
  const policyAuthority =
    evidencePolicy === "legacy-v1"
      ? (reviewedPolicy(policySha, api(`compare/${policySha}...main`), api(`commits/${policySha}`)),
        legacyAuthority(policySha))
      : (reviewedPolicy(policySha, api(`compare/${policySha}...main`), api(`commits/${policySha}`)),
        sourceAuthority(policySha, gitBlob));
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
  assertRecoveryReceiptPolicy(receipt, run, expected);
  same(
    receipt.recovery,
    { run: recoveryRun, attempt, policySha },
    "replacement run/attempt/source mismatch",
  );
  same(receipt.authority, policyAuthority, "replacement policy changed");
  same(receipt.files, files(directory), "complete raw replacement observations changed");
  const integration = (name: string, publication: string) =>
    integrationIdentity(
      verifyStoredIntegration(
        object(object(JSON.parse(readFileSync(path.join(directory, publication), "utf8"))).github),
        expected,
        JSON.parse(readFileSync(path.join(directory, name), "utf8")),
      ),
    );
  same(
    integration("integration.json", "publication-before.json"),
    integration("integration-after.json", "publication-after.json"),
    "original signed integration changed after requalification",
  );
  const original = JSON.parse(
    readFileSync(path.join(directory, "original.json"), "utf8"),
  ) as Original;
  same(original.identity, expected, "original identity reused");
  const admitted = evidencePolicy === "legacy-v1" ? legacyAdmission(original) : admission(original);
  if (evidencePolicy === "current-v2") {
    verifyOriginalLogDirectory(directory, "original-log-capture", original);
    verifyOriginalLogDirectory(directory, "final-log-capture", original);
  }
  for (const role of roles) {
    const consumerDirectory = path.join(directory, "consumers", role);
    const packet =
      evidencePolicy === "legacy-v1"
        ? object(JSON.parse(readFileSync(path.join(consumerDirectory, "consumer.json"), "utf8")))
        : verifyConsumerDirectory(consumerDirectory, original);
    if (evidencePolicy === "legacy-v1") verifyLegacyConsumer(packet, original);
    (evidencePolicy === "legacy-v1" ? assertLegacyOracleFiles : assertOracleFiles)(
      path.join(directory, "consumers", role),
      original.policy,
      role === "deno-linux",
    );
    await revalidatePublicArchives(
      packet.packages as InstalledArchive[],
      object(object(packet.installed).native) as { path: string; sha256: string },
    );
    same(packet.role, role, "replacement role substituted");
    consumerAttempt(packet.run, recoveryRun, attempt, policySha);
  }
  const live =
    evidencePolicy === "legacy-v1"
      ? await collectLegacyOriginal(expected, policySha)
      : await collectOriginalSnapshot(expected, policySha, policyAuthority);
  for (const key of [
    "policy",
    "run",
    "jobs",
    "failed",
    "log",
    "logIdentity",
    ...(evidencePolicy === "legacy-v1" ? [] : ["logCustody"]),
    "blobs",
    "artifacts",
    "pin",
    "sourceReceipt",
  ])
    same(original[key], live[key], `original ${key} changed after requalification`);
  const currentIntegration = verifyIntegration(
    object(
      object(JSON.parse(readFileSync(path.join(directory, "publication-after.json"), "utf8")))
        .github,
    ),
    expected,
  );
  same(
    integrationIdentity(currentIntegration),
    integration("integration-after.json", "publication-after.json"),
    "original signed integration changed since requalification",
  );
  same(api(`actions/runs/${recoveryRun}`), run, "replacement attempt changed during verification");
  assert.ok(realpathSync(directory) === directory, "replacement root redirected");
  return { admission: admitted, receipt, directory, integration: currentIntegration };
}
