import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { appendFileSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  admitV1ExactCandidate,
  admitV1ExactCandidateHost,
  buildV1ExactCandidateHostPlan,
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
  const receipts = option(args, "--receipts");
  const output = option(args, "--output");
  const matrix = buildV1SupportMatrix();
  const bundle = readJson<V1ExactCandidateReceiptBundle>(receipts);
  writeJson(output, admitV1ExactCandidateHost(matrix, bundle, candidate, host));
}

function admissionFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
    .map((entry) => path.join(directory, entry.name))
    .toSorted((left, right) => left.localeCompare(right));
}

function admit(args: readonly string[]): void {
  const candidate = option(args, "--candidate");
  const admissionsDirectory = option(args, "--admissions");
  const output = option(args, "--output");
  const admissions = admissionFiles(admissionsDirectory).map((filename) =>
    readJson<V1ExactCandidateHostAdmission>(filename),
  );
  const admitted = admitV1ExactCandidate(buildV1SupportMatrix(), candidate, admissions);
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
  default:
    throw new Error(
      "Usage: node tools/commands/release/v1-exact-candidate.ts <plan|validate-host|admit> [options]",
    );
}
