import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const commitPattern = /^[0-9a-f]{40}$/;
export const hashPattern = /^[0-9a-f]{64}$/;
export const projectPattern = /^[a-z0-9][a-z0-9-]*$/;
export const requiredStages = ["parse", "lint", "output"] as const;
export const totalFields = [
  "patinaFindingCount",
  "baselineFindingCount",
  "comparableBaselineCount",
  "sharedCount",
  "messageDifferenceCount",
  "documentedDivergenceCount",
  "ruleLocationDivergenceCount",
  "falsePositiveCount",
  "falseNegativeCount",
  "unimplementedCount",
  "intentionalDivergenceCount",
  "patinaOnlyRuleFindingCount",
  "baselineParseErrorCount",
  "baselineInvalidRangeCount",
] as const;
export const summaryFields = [
  ...totalFields,
  "patinaOutsideBaselineSurfaceCount",
  "referenceCoverageIntegrityErrorCount",
  "baselineExcludedNonVueCount",
] as const;
export const parityFields = [
  "falsePositiveCount",
  "falseNegativeCount",
  "messageDifferenceCount",
  "documentedDivergenceCount",
  "ruleLocationDivergenceCount",
  "unimplementedCount",
  "intentionalDivergenceCount",
  "patinaOnlyRuleFindingCount",
  "patinaOutsideBaselineSurfaceCount",
  "referenceCoverageIntegrityErrorCount",
  "baselineParseErrorCount",
  "baselineInvalidRangeCount",
  "baselineExcludedNonVueCount",
] as const;
const arrayCountFields = [
  ["shared", "sharedCount"],
  ["messageDifferences", "messageDifferenceCount"],
  ["documentedDivergences", "documentedDivergenceCount"],
  ["ruleLocationDivergences", "ruleLocationDivergenceCount"],
  ["falsePositives", "falsePositiveCount"],
  ["falseNegatives", "falseNegativeCount"],
  ["unimplemented", "unimplementedCount"],
  ["intentionalDivergences", "intentionalDivergenceCount"],
  ["patinaOnlyRuleFindings", "patinaOnlyRuleFindingCount"],
  ["patinaOutsideBaselineSurface", "patinaOutsideBaselineSurfaceCount"],
] as const;

export type JsonRecord = Record<string, unknown>;
export type TotalField = (typeof totalFields)[number];

export interface RegistryProject extends JsonRecord {
  id: string;
  fixturePath: string;
  revision?: string | null;
  coverage: string[];
  vueGlobs: string[];
  expectedVueFileCount?: number | null;
}

export interface RunEvidence extends JsonRecord {
  commitSha: string;
  runtime: JsonRecord;
  machine: JsonRecord;
}

export interface BuildReceipt extends JsonRecord {
  schema: string;
  version: number;
  recipe: string;
  sourceRevision: string;
  binaryPath: string;
  binarySha256: string;
  cliVersion: string;
}

export interface LintSummary extends JsonRecord {
  referenceCoverageIntegrityErrorCount: number;
  patinaFindingCount: number;
  baselineFindingCount: number;
  comparableBaselineCount: number;
  sharedCount: number;
  messageDifferenceCount: number;
  documentedDivergenceCount: number;
  ruleLocationDivergenceCount: number;
  falsePositiveCount: number;
  falseNegativeCount: number;
  unimplementedCount: number;
  intentionalDivergenceCount: number;
  patinaOnlyRuleFindingCount: number;
  patinaOutsideBaselineSurfaceCount: number;
  baselineParseErrorCount: number;
  baselineExcludedNonVueCount: number;
  baselineInvalidRangeCount: number;
  falsePositiveRatio: number;
  falseNegativeRatio: number;
}

export interface LintBudget extends JsonRecord {
  maxFalsePositiveCount: number;
  maxFalseNegativeCount: number;
  falsePositivePassed: boolean;
  falseNegativePassed: boolean;
  unusableReason: string | null;
  verdict: "passed" | "breached" | "unusable";
  passed: boolean;
}

export interface LintDivergence extends JsonRecord {
  schema: string;
  version: number;
  project: string;
  upstream: { package: string; version: string };
  summary: LintSummary;
  referenceCoverageExpectations: Array<{ passed?: boolean } & JsonRecord>;
  shared: unknown[];
  messageDifferences: unknown[];
  documentedDivergences: unknown[];
  ruleLocationDivergences: unknown[];
  falsePositives: unknown[];
  falseNegatives: unknown[];
  unimplemented: unknown[];
  intentionalDivergences: unknown[];
  patinaOnlyRuleFindings: unknown[];
  patinaOutsideBaselineSurface: unknown[];
  sha256: string;
}

export interface LintProjectReport extends JsonRecord {
  schema: string;
  version: number;
  project: string;
  revision: string | null;
  preset: string;
  evidence: RunEvidence;
  files: { comparedCount: number; expectedCount: number | null };
  baseline: {
    package: string;
    version: string;
    runtimeVersions: JsonRecord;
    comparedRuleCount: number;
    mappedRuleCount: number;
    skippedByPresetCount: number;
    parseErrorResults: unknown[];
  } & JsonRecord;
  raw: { patinaResults: unknown[]; patinaFindings: unknown[]; eslintResults: unknown[] };
  divergence: LintDivergence;
  budget: LintBudget;
}

export interface LintIndex extends JsonRecord {
  schema: string;
  version: number;
  evidence: RunEvidence;
  preset: string;
  projectCount: number;
  budget: JsonRecord;
  totals: Record<TotalField, number>;
  projects: Array<{ project: string } & LintSummary>;
}

export interface LinterProjectEvidence extends JsonRecord {
  id: string;
  fixturePath: string;
  revision: string | null;
  expectedFileCount: number | null;
  reportedExpectedFileCount: number | null;
  comparedFileCount: number;
  unmatchedPatterns: string[];
  droppedConfigMessageCount: number;
  preset: string;
  matrixRunSha256: string;
  rawRunSha256: string;
  reportSha256: string;
  divergenceSha256: string;
  budget: Pick<LintBudget, "verdict" | "passed" | "unusableReason">;
  summary: LintSummary;
}

export interface Blocker extends JsonRecord {
  code: string;
  detail: string;
}

export interface LinterDeletionArtifact extends JsonRecord {
  schema: string;
  version: number;
  sourceRevision: string;
  target: "lint";
  registry: { path: string; sha256: string };
  shard: { index: number; count: number };
  buildReceiptSha256: string;
  binarySha256: string;
  requiredStages: readonly string[];
  measurement: { preset: "ecosystem"; measureCoverageGap: true };
  inputs: {
    matrix: { path: "summary.json"; sha256: string };
    divergenceIndex: { path: "lint-divergence-summary.json"; sha256: string };
    ruleMap: { path: "tests/_fixtures/patina-eslint-vue-rule-map.json"; sha256: string };
    divergenceLedger: {
      path: "tests/_fixtures/patina-lint-documented-divergences.json";
      sha256: string;
    };
  };
  nativeRouteReceipt: null;
  actionsCustodyReceipt: null;
  candidateEvidenceReceipt: null;
  stabilityHistoryReceipt: null;
  fixtureProjectMembershipReceipt: null;
  projects: LinterProjectEvidence[];
  totals: {
    projectCount: number;
    comparedFileCount: number;
    falsePositiveCount: number;
    falseNegativeCount: number;
    messageDifferenceCount: number;
  };
  status: "blocked";
  blockers: Blocker[];
  sha256: string;
}

export function asRecord(value: unknown, label: string): JsonRecord {
  assert(value && typeof value === "object" && !Array.isArray(value), `${label} is required`);
  return value as JsonRecord;
}

export function integer(value: unknown, label: string): number {
  assert(Number.isSafeInteger(value) && Number(value) >= 0, `${label} must be non-negative`);
  return Number(value);
}

export function exactKeys(value: JsonRecord, expected: string[], label: string): void {
  assert.deepEqual(
    Object.keys(value).sort(byteOrder),
    [...expected].sort(byteOrder),
    `${label} fields drifted`,
  );
}

function byteOrder(left: string, right: string): number {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

function canonical(value: unknown, key = ""): string {
  if (typeof value === "bigint") return String(value);
  if (Array.isArray(value)) return `[${value.map((item) => canonical(item)).join(",")}]`;
  if (value && typeof value === "object") {
    const object = value as JsonRecord;
    return `{${Object.keys(object)
      .sort(byteOrder)
      .map((name) => `${JSON.stringify(name)}:${canonical(object[name], name)}`)
      .join(",")}}`;
  }
  if (typeof value === "number" && ["falsePositiveRatio", "falseNegativeRatio"].includes(key)) {
    if (Object.is(value, -0)) return "-0.0";
    const magnitude = Math.abs(value);
    if (magnitude !== 0 && (magnitude < 0.00001 || magnitude >= 1e16)) {
      return value.toExponential();
    }
    return Number.isInteger(value) ? `${value}.0` : String(value);
  }
  const encoded = JSON.stringify(value);
  assert.notEqual(encoded, undefined, "canonical JSON cannot contain undefined");
  return encoded;
}

export function sha256(value: string | Buffer): string {
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value);
  return createHash("sha256").update(bytes).digest("hex");
}

export function canonicalSha256(value: unknown): string {
  return sha256(canonical(value));
}

export function validateBuildReceipt(
  value: unknown,
  sourceRevision: string,
): { receipt: BuildReceipt; sha256: string } {
  const record = asRecord(value, "linter source-build receipt");
  exactKeys(
    record,
    ["schema", "version", "recipe", "sourceRevision", "binaryPath", "binarySha256", "cliVersion"],
    "linter source-build receipt",
  );
  const receipt = record as BuildReceipt;
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.recipe, "cargo build --profile ci -p vize");
  assert.equal(receipt.sourceRevision, sourceRevision);
  assert.match(receipt.binaryPath, /^target\/ci\/vize(?:\.exe)?$/);
  assert.match(receipt.binarySha256, hashPattern);
  assert.match(receipt.cliVersion, /^vize \d+\.\d+\.\d+-dev$/);
  return { receipt, sha256: canonicalSha256(receipt) };
}

export function exactIds(actual: string[], expected: string[], label: string): void {
  assert.deepEqual(
    [...actual].sort(byteOrder),
    [...expected].sort(byteOrder),
    `${label} does not match the selected real-project shard`,
  );
}

export function validateSummary(value: unknown, label: string): LintSummary {
  const summary = asRecord(value, label);
  const expected = [...summaryFields, "falsePositiveRatio", "falseNegativeRatio"].sort(byteOrder);
  exactKeys(summary, expected, label);
  const result: JsonRecord = {};
  for (const field of summaryFields) result[field] = integer(summary[field], `${label}.${field}`);
  for (const field of ["falsePositiveRatio", "falseNegativeRatio"] as const) {
    const ratio = summary[field];
    assert(
      typeof ratio === "number" && Number.isFinite(ratio) && ratio >= 0 && ratio <= 1,
      `${label}.${field} must be a finite ratio`,
    );
    result[field] = ratio;
  }
  const typed = result as LintSummary;
  const comparablePatinaCount =
    typed.sharedCount + typed.falsePositiveCount + typed.ruleLocationDivergenceCount;
  assert.equal(
    typed.patinaFindingCount,
    comparablePatinaCount +
      typed.patinaOnlyRuleFindingCount +
      typed.patinaOutsideBaselineSurfaceCount,
    `${label}.patinaFindingCount is not a complete partition`,
  );
  assert.equal(
    typed.comparableBaselineCount,
    typed.sharedCount + typed.falseNegativeCount + typed.ruleLocationDivergenceCount,
    `${label}.comparableBaselineCount is not a complete partition`,
  );
  assert.equal(
    typed.baselineFindingCount,
    typed.comparableBaselineCount + typed.unimplementedCount + typed.intentionalDivergenceCount,
    `${label}.baselineFindingCount is not a complete partition`,
  );
  assert(
    typed.messageDifferenceCount <= typed.sharedCount,
    `${label}.messageDifferenceCount exceeds sharedCount`,
  );
  assert.equal(
    typed.falsePositiveRatio,
    comparablePatinaCount === 0 ? 0 : typed.falsePositiveCount / comparablePatinaCount,
    `${label}.falsePositiveRatio drifted`,
  );
  assert.equal(
    typed.falseNegativeRatio,
    typed.comparableBaselineCount === 0
      ? 0
      : typed.falseNegativeCount / typed.comparableBaselineCount,
    `${label}.falseNegativeRatio drifted`,
  );
  return typed;
}

export function lintDivergenceSha256(divergence: JsonRecord): string {
  const payload = Object.fromEntries(
    Object.entries(divergence).filter(
      ([key]) => !["schema", "version", "project", "upstream", "sha256"].includes(key),
    ),
  );
  return canonicalSha256(payload);
}

export function validateDivergence(
  value: unknown,
  projectId: string,
): { divergence: LintDivergence; summary: LintSummary } {
  asRecord(value, `${projectId} lint divergence`);
  const divergence = value as LintDivergence;
  exactKeys(
    divergence,
    [
      "schema",
      "version",
      "project",
      "upstream",
      "summary",
      "referenceCoverageExpectations",
      "shared",
      "messageDifferences",
      "documentedDivergences",
      "ruleLocationDivergences",
      "falsePositives",
      "falseNegatives",
      "unimplemented",
      "intentionalDivergences",
      "patinaOnlyRuleFindings",
      "patinaOutsideBaselineSurface",
      "sha256",
    ],
    `${projectId}.divergence`,
  );
  assert.equal(divergence.schema, "vize.fixtureLintDivergence");
  assert.equal(divergence.version, 1);
  assert.equal(divergence.project, projectId);
  assert.equal(divergence.upstream?.package, "eslint-plugin-vue");
  assert.equal(typeof divergence.upstream?.version, "string");
  assert(divergence.upstream.version.length > 0);
  const summary = validateSummary(divergence.summary, `${projectId}.divergence.summary`);
  for (const [field, count] of arrayCountFields) {
    assert(Array.isArray(divergence[field]), `${projectId}.${field} must be an array`);
    assert.equal(divergence[field].length, summary[count], `${projectId}.${field} count drifted`);
  }
  assert(Array.isArray(divergence.referenceCoverageExpectations));
  assert.equal(
    divergence.referenceCoverageExpectations.filter((item) => item.passed !== true).length,
    summary.referenceCoverageIntegrityErrorCount,
  );
  assert.match(divergence.sha256, hashPattern);
  assert.equal(
    divergence.sha256,
    lintDivergenceSha256(divergence),
    `${projectId}: lint divergence digest mismatch`,
  );
  return { divergence, summary };
}

export function validateBudget(
  value: unknown,
  summary: LintSummary,
  projectId: string,
): LintBudget {
  const budgetRecord = asRecord(value, `${projectId}.budget`);
  const keys = [
    "falseNegativePassed",
    "falsePositivePassed",
    "maxFalseNegativeCount",
    "maxFalsePositiveCount",
    "passed",
    "unusableReason",
    "verdict",
  ].sort(byteOrder);
  assert.deepEqual(Object.keys(budgetRecord).sort(byteOrder), keys);
  const budget = value as LintBudget;
  assert.equal(budget.maxFalsePositiveCount, 0);
  assert.equal(budget.maxFalseNegativeCount, 0);
  assert.equal(budget.falsePositivePassed, summary.falsePositiveCount === 0);
  assert.equal(budget.falseNegativePassed, summary.falseNegativeCount === 0);
  assert(budget.unusableReason === null || typeof budget.unusableReason === "string");
  const verdict =
    budget.unusableReason !== null
      ? "unusable"
      : budget.falsePositivePassed && budget.falseNegativePassed
        ? "passed"
        : "breached";
  assert.equal(budget.verdict, verdict);
  assert.equal(budget.passed, verdict === "passed");
  return budget;
}
