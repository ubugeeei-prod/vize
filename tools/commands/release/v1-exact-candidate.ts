import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { appendFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  admitV1ExactCandidate,
  admitV1ExactCandidateHost,
  buildV1ExactCandidateHostPlan,
  V1_EXACT_CANDIDATE_BUNDLE_SCHEMA,
  type V1ExactCandidateHostAdmission,
  type V1ExactCandidateReceiptBundle,
} from "../../support/compat/v1/exact-candidate.ts";
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

function positiveIntegerOption(args: readonly string[], name: string): number {
  const value = option(args, name);
  if (!/^[1-9][0-9]*$/u.test(value)) throw new Error(`${name} must be a positive integer`);
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) throw new Error(`${name} must be a safe integer`);
  return parsed;
}

function plan(args: readonly string[]): void {
  const candidate = option(args, "--candidate");
  assert.match(candidate, /^[0-9a-f]{40}$/u, "candidate must be one exact 40-character SHA");
  const hosts = buildV1ExactCandidateHostPlan(buildV1SupportMatrix());
  const rowIds = hosts.flatMap(({ rowIds: ids }) => ids).toSorted();
  const receipt = {
    schema: "vize.v1-exact-candidate-plan.v1",
    candidateSource: candidate,
    counts: { hostPlans: hosts.length, requiredRows: rowIds.length },
    rowInventorySha256: createHash("sha256").update(JSON.stringify(rowIds)).digest("hex"),
    hosts,
  };
  const githubOutput = args.includes("--github-output") ? process.env.GITHUB_OUTPUT : undefined;
  if (githubOutput) {
    appendFileSync(githubOutput, `hosts=${JSON.stringify({ include: hosts })}\n`);
    appendFileSync(githubOutput, `host_count=${hosts.length}\n`);
    appendFileSync(
      githubOutput,
      `row_count=${hosts.reduce((count, host) => count + host.rowIds.length, 0)}\n`,
    );
  }
  const output = optionalOption(args, "--output");
  if (output) writeJson(output, receipt);
  if (!githubOutput && !output) console.log(JSON.stringify(receipt, null, 2));
}

function validateHost(args: readonly string[]): void {
  const candidate = option(args, "--candidate");
  const host = option(args, "--host");
  const run = positiveIntegerOption(args, "--run");
  const runAttempt = positiveIntegerOption(args, "--run-attempt");
  const receipts = option(args, "--receipts");
  const output = option(args, "--output");
  const matrix = buildV1SupportMatrix();
  const bundle = readJson<V1ExactCandidateReceiptBundle>(receipts);
  writeJson(
    output,
    admitV1ExactCandidateHost(matrix, bundle, candidate, host, { run, runAttempt }),
  );
}

function jsonFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
    .map((entry) => path.join(directory, entry.name))
    .toSorted((left, right) => left.localeCompare(right));
}

function admit(args: readonly string[]): void {
  const candidate = option(args, "--candidate");
  const admissionsDirectory = option(args, "--admissions");
  const output = option(args, "--output");
  const run = positiveIntegerOption(args, "--run");
  const runAttempt = positiveIntegerOption(args, "--run-attempt");
  const admissions = jsonFiles(admissionsDirectory).map((filename) =>
    readJson<V1ExactCandidateHostAdmission>(filename),
  );
  const admitted = admitV1ExactCandidate(buildV1SupportMatrix(), candidate, admissions, {
    run,
    runAttempt,
  });
  if (args.includes("--enforce-complete"))
    assert.equal(
      admitted.counts.completeRequiredRows,
      admitted.counts.requiredRows,
      `exact candidate admitted ${admitted.counts.completeRequiredRows}/${admitted.counts.requiredRows} required rows`,
    );
  writeJson(output, admitted);
  console.log(
    `Admitted ${admitted.counts.completeRequiredRows}/${admitted.counts.requiredRows} required rows for ${candidate}`,
  );
}

// Admits every host row of one run from the receipt bundles its lanes uploaded. A run
// without bundles admits nothing, so every row stays a gap; an invalid or foreign
// receipt fails even when completeness is not enforced. The manifest is written before
// completeness is enforced, so a failing run still keeps what it admitted.
function admitRun(args: readonly string[]): void {
  const candidate = option(args, "--candidate");
  const receiptsDirectory = option(args, "--receipts");
  const output = option(args, "--output");
  const runIdentity = {
    run: positiveIntegerOption(args, "--run"),
    runAttempt: positiveIntegerOption(args, "--run-attempt"),
  };
  const files = existsSync(receiptsDirectory) ? jsonFiles(receiptsDirectory) : [];
  const receipts = files.flatMap((filename) => {
    const bundle = readJson<V1ExactCandidateReceiptBundle>(filename);
    assert.equal(bundle.schema, V1_EXACT_CANDIDATE_BUNDLE_SCHEMA, `${filename}: schema`);
    return bundle.receipts;
  });
  const bundle = { schema: V1_EXACT_CANDIDATE_BUNDLE_SCHEMA, receipts };
  const matrix = buildV1SupportMatrix();
  const admissions = buildV1ExactCandidateHostPlan(matrix).map(({ id }) =>
    admitV1ExactCandidateHost(matrix, bundle, candidate, id, runIdentity),
  );
  const admitted = admitV1ExactCandidate(matrix, candidate, admissions, runIdentity);
  writeJson(output, admitted);
  const { completeRequiredRows, requiredRows } = admitted.counts;
  console.log(
    `Admitted ${completeRequiredRows}/${requiredRows} required rows for ${candidate} from ${files.length} receipt bundles`,
  );
  if (args.includes("--enforce-complete"))
    assert.equal(
      completeRequiredRows,
      requiredRows,
      `exact candidate admitted ${completeRequiredRows}/${requiredRows} required rows`,
    );
}

const [command, ...args] = process.argv.slice(2);
switch (command) {
  case "plan":
    plan(args);
    break;
  case "validate-host":
    validateHost(args);
    break;
  case "admit":
    admit(args);
    break;
  case "admit-run":
    admitRun(args);
    break;
  default:
    throw new Error(
      "Usage: node tools/commands/release/v1-exact-candidate.ts <plan|validate-host|admit|admit-run> [options]",
    );
}
