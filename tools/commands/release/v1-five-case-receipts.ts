import assert from "node:assert/strict";
import { appendFileSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  admitV1FiveCaseReceipts,
  buildV1FiveCaseShardPlan,
  validateV1FiveCaseReceiptBundles,
  type V1FiveCaseMode,
  type V1FiveCaseReceiptBundle,
} from "../../support/compat/v1/five-case-receipts.ts";
import { buildV1SupportMatrix } from "../../support/compat/v1/support-matrix.ts";

function readJson<T>(filename: string): T {
  return JSON.parse(readFileSync(filename, "utf8")) as T;
}

function writeJson(filename: string, value: unknown): void {
  writeFileSync(filename, `${JSON.stringify(value, null, 2)}\n`);
}

function option(args: readonly string[], name: string): string {
  const index = args.indexOf(name);
  const value = index < 0 ? undefined : args[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`${name} is required`);
  return value;
}

function optionalOption(args: readonly string[], name: string): string | undefined {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  const value = args[index + 1];
  if (!value || value.startsWith("--")) throw new Error(`${name} requires a value`);
  return value;
}

function positiveInteger(args: readonly string[], name: string): number {
  const raw = option(args, name);
  const value = Number(raw);
  assert.ok(Number.isSafeInteger(value) && value > 0, `${name} must be a positive integer`);
  return value;
}

function exactSha(args: readonly string[], name: string): string {
  const value = option(args, name);
  assert.match(value, /^[0-9a-f]{40}$/u, `${name} must be one exact 40-character SHA`);
  return value;
}

function mode(args: readonly string[]): V1FiveCaseMode {
  const value = option(args, "--mode");
  assert.ok(value === "pr" || value === "release", "--mode must be pr or release");
  return value;
}

function githubOutput(entries: Record<string, string | number>): void {
  const filename = process.env.GITHUB_OUTPUT;
  if (!filename) return;
  for (const [name, value] of Object.entries(entries)) appendFileSync(filename, `${name}=${value}\n`);
}

function plan(args: readonly string[]): void {
  const sourceSha = exactSha(args, "--source");
  const candidateSha = exactSha(args, "--candidate");
  const selectedMode = mode(args);
  const runId = positiveInteger(args, "--run");
  const runAttempt = positiveInteger(args, "--attempt");
  const shardTotal = positiveInteger(args, "--shards");
  const planReceipt = {
    ...buildV1FiveCaseShardPlan(buildV1SupportMatrix(), selectedMode, shardTotal),
    sourceSha,
    candidateSha,
    runId,
    runAttempt,
  };
  const output = optionalOption(args, "--output");
  if (output) writeJson(output, planReceipt);
  else console.log(JSON.stringify(planReceipt, null, 2));
  githubOutput({
    registry_digest: planReceipt.registrySha256,
    selector_count: planReceipt.counts.selectors,
    shard_count: planReceipt.counts.shards,
    shards: JSON.stringify({ include: planReceipt.shards }),
  });
}

function bundleFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
    .map((entry) => path.join(directory, entry.name))
    .toSorted((left, right) => left.localeCompare(right));
}

function readBundles(directory: string): V1FiveCaseReceiptBundle[] {
  const files = bundleFiles(directory);
  assert.ok(files.length > 0, `no receipt bundles found in ${directory}`);
  return files.map((filename) => readJson<V1FiveCaseReceiptBundle>(filename));
}

function validateBundles(args: readonly string[]): void {
  const matrix = buildV1SupportMatrix();
  const bundles = readBundles(option(args, "--bundles"));
  const identity = {
    sourceSha: exactSha(args, "--source"),
    candidateSha: exactSha(args, "--candidate"),
    mode: mode(args),
  };
  validateV1FiveCaseReceiptBundles(matrix, bundles);
  for (const bundle of bundles) {
    assert.equal(bundle.sourceSha, identity.sourceSha, `${bundle.id}: source SHA`);
    assert.equal(bundle.candidateSha, identity.candidateSha, `${bundle.id}: candidate SHA`);
    assert.equal(bundle.mode, identity.mode, `${bundle.id}: mode`);
  }
  const plan = buildV1FiveCaseShardPlan(matrix, identity.mode, 1);
  githubOutput({
    registry_digest: plan.registrySha256,
    selector_count: plan.counts.selectors,
  });
  console.log(`Validated ${bundles.length} five-case receipt bundles`);
}

function admit(args: readonly string[]): void {
  const identity = {
    sourceSha: exactSha(args, "--source"),
    candidateSha: exactSha(args, "--candidate"),
    mode: mode(args),
  };
  const output = option(args, "--output");
  const admitted = admitV1FiveCaseReceipts(
    buildV1SupportMatrix(),
    identity,
    readBundles(option(args, "--bundles")),
  );
  if (args.includes("--enforce-complete"))
    assert.equal(
      admitted.counts.admittedCaseSelectors,
      admitted.counts.caseSelectors,
      `five-case admission incomplete: ${admitted.counts.admittedCaseSelectors}/${admitted.counts.caseSelectors}; missing=${admitted.counts.missingCaseSelectors}, skipped=${admitted.counts.skippedCaseSelectors}, stale=${admitted.counts.staleCaseSelectors}, unsupported=${admitted.counts.unsupportedCaseSelectors}`,
    );
  writeJson(output, admitted);
  console.log(
    `Admitted ${admitted.counts.admittedCaseSelectors}/${admitted.counts.caseSelectors} five-case selectors`,
  );
  githubOutput({
    registry_digest: admitted.fiveCaseAdmission.registrySha256,
    bundle_digest: admitted.fiveCaseAdmission.bundleSha256,
    selector_count: admitted.counts.caseSelectors,
    admitted_count: admitted.counts.admittedCaseSelectors,
  });
}

const [command, ...args] = process.argv.slice(2);
switch (command) {
  case "plan":
    plan(args);
    break;
  case "validate-bundles":
    validateBundles(args);
    break;
  case "admit":
    admit(args);
    break;
  default:
    throw new Error(
      "Usage: node tools/commands/release/v1-five-case-receipts.ts <plan|validate-bundles|admit> [options]",
    );
}
