import assert from "node:assert/strict";

import {
  canonicalJson,
  semanticCategories,
  semanticNormalization,
  sha256,
} from "./syntax-semantic-divergence.ts";
import { assertNormalizedPath, byteOrder } from "./syntax-evidence.ts";
import { shikiVueOracleProvenance } from "./shiki-vue-oracle.ts";
import { textmateDependencyVersions } from "./textmate-deps.ts";

export type ProjectArtifact = {
  comparisonSha256: string;
  documentedDivergences: unknown[];
  falseNegatives: unknown[];
  falsePositives: unknown[];
  fileCount: number;
  id: string;
  inputSha256: string;
  revision: string;
  semanticSpanCount: number;
};

type DivergenceArtifactInput = {
  commitSha: string;
  grammars: unknown;
  ledger: { path: string; sha256: string };
  projects: ProjectArtifact[];
  shard: { count: number | null; index: number | null };
};

type DivergenceSummary = {
  documentedDivergenceCount: number;
  falseNegativeCount: number;
  falsePositiveCount: number;
  fileCount: number;
  projectCount: number;
  semanticSpanCount: number;
};

export type DivergenceArtifact = {
  commitSha: string;
  grammars: unknown;
  ledger: DivergenceArtifactInput["ledger"];
  mode: "record-only";
  normalization: typeof semanticNormalization;
  projects: ProjectArtifact[];
  schema: "vize.fixtureSyntaxHighlighterDivergence";
  sha256: string;
  shard: DivergenceArtifactInput["shard"];
  summary: DivergenceSummary;
  version: 1;
};

export function createDivergenceArtifact(input: DivergenceArtifactInput): DivergenceArtifact {
  const summary = summarize(input.projects);
  const body: Omit<DivergenceArtifact, "sha256"> = {
    schema: "vize.fixtureSyntaxHighlighterDivergence",
    version: 1,
    mode: "record-only",
    commitSha: input.commitSha,
    shard: input.shard,
    normalization: semanticNormalization,
    grammars: input.grammars,
    ledger: input.ledger,
    summary,
    projects: input.projects,
  };
  const artifact: DivergenceArtifact = { ...body, sha256: sha256(canonicalJson(body)) };
  validateDivergenceArtifact(artifact);
  return artifact;
}

export function validateDivergenceArtifact(value: unknown): asserts value is DivergenceArtifact {
  assert.ok(value != null && typeof value === "object" && !Array.isArray(value));
  const artifact = value as Record<string, any>;
  assert.deepEqual(Object.keys(artifact), [
    "schema",
    "version",
    "mode",
    "commitSha",
    "shard",
    "normalization",
    "grammars",
    "ledger",
    "summary",
    "projects",
    "sha256",
  ]);
  assert.equal(artifact.schema, "vize.fixtureSyntaxHighlighterDivergence");
  assert.equal(artifact.version, 1);
  assert.equal(artifact.mode, "record-only");
  assert.match(artifact.commitSha, /^[0-9a-f]{40}$/);
  assert.deepEqual(artifact.normalization, semanticNormalization);
  validateShard(artifact.shard);
  validateGrammars(artifact.grammars);
  assertRecord(artifact.ledger, "artifact ledger");
  assert.deepEqual(
    Object.keys(artifact.ledger),
    ["path", "sha256"],
    "unexpected artifact ledger fields",
  );
  assert.match(artifact.ledger?.sha256, /^[0-9a-f]{64}$/);
  assertNormalizedPath(artifact.ledger?.path, "artifact path");
  assert.ok(Array.isArray(artifact.projects) && artifact.projects.length > 0);
  const projectIds = new Set<string>();
  for (const project of artifact.projects) {
    validateProject(project);
    assert.ok(!projectIds.has(project.id), `duplicate artifact project ${project.id}`);
    projectIds.add(project.id);
  }
  assert.deepEqual(
    artifact.summary,
    summarize(artifact.projects),
    "syntax divergence artifact summary mismatch",
  );
  const { sha256: digest, ...body } = artifact;
  assert.equal(digest, sha256(canonicalJson(body)), "syntax divergence artifact digest mismatch");
}

export function renderDivergenceMarkdown(artifact: DivergenceArtifact): string {
  const { summary } = artifact;
  return `${[
    "## Syntax highlighter semantic divergence",
    "",
    `Commit: ${artifact.commitSha}`,
    `Mode: ${artifact.mode}`,
    `Projects: ${summary.projectCount}`,
    `Vue files: ${summary.fileCount}`,
    `Semantic spans: ${summary.semanticSpanCount}`,
    `False positives: ${summary.falsePositiveCount}`,
    `False negatives: ${summary.falseNegativeCount}`,
    `Documented divergences: ${summary.documentedDivergenceCount}`,
    `Digest: ${artifact.sha256}`,
    "",
  ].join("\n")}`;
}

function validateProject(project: Record<string, any>): void {
  assert.deepEqual(Object.keys(project), [
    "id",
    "revision",
    "fileCount",
    "inputSha256",
    "semanticSpanCount",
    "comparisonSha256",
    "falsePositives",
    "falseNegatives",
    "documentedDivergences",
  ]);
  assert.match(project.id, /^[a-z0-9][a-z0-9-]*$/);
  assert.match(project.revision, /^[0-9a-f]{40}$/);
  assert.match(project.inputSha256, /^[0-9a-f]{64}$/);
  assert.match(project.comparisonSha256, /^[0-9a-f]{64}$/);
  assert.ok(Number.isSafeInteger(project.fileCount) && project.fileCount >= 0);
  assert.ok(Number.isSafeInteger(project.semanticSpanCount) && project.semanticSpanCount >= 0);
  for (const key of ["falsePositives", "falseNegatives", "documentedDivergences"]) {
    assert.ok(Array.isArray(project[key]));
    for (const record of project[key]) {
      validateRecord(record, key, project.id);
    }
  }
}

function validateShard(value: unknown): void {
  assertRecord(value, "artifact shard");
  const shard = value as Record<string, unknown>;
  assert.deepEqual(Object.keys(shard), ["count", "index"]);
  if (shard.count === null || shard.index === null) {
    assert.equal(shard.count, null, "unsharded artifact count must be null");
    assert.equal(shard.index, null, "unsharded artifact index must be null");
    return;
  }
  assert.ok(
    typeof shard.count === "number" && Number.isSafeInteger(shard.count) && shard.count > 0,
    "artifact shard count must be a positive integer",
  );
  assert.ok(
    typeof shard.index === "number" &&
      Number.isSafeInteger(shard.index) &&
      shard.index >= 0 &&
      shard.index < shard.count,
    "artifact shard index must be within count",
  );
}

function validateGrammars(value: unknown): void {
  assertRecord(value, "artifact grammars");
  const grammars = value as Record<string, any>;
  assert.deepEqual(Object.keys(grammars), ["oracle", "vize"]);
  const oracle = grammars.oracle;
  assertRecord(oracle, "oracle grammar evidence");
  assert.deepEqual(Object.keys(oracle), [
    "configuredGrammarSha256",
    "dependencyVersions",
    "grammarClosureSha256",
    "licenseSha256",
    "module",
    "moduleSha256",
    "package",
    "requestedScopes",
    "rootScope",
    "unresolvedScopeSentinels",
    "version",
  ]);
  assert.match(oracle.configuredGrammarSha256, /^[0-9a-f]{64}$/);
  assert.deepEqual(oracle.dependencyVersions, textmateDependencyVersions);
  for (const [key, expected] of Object.entries(shikiVueOracleProvenance)) {
    assert.equal(oracle[key], expected, `oracle ${key} provenance drifted`);
  }
  assert.equal(oracle.rootScope, "text.html.vue");
  assertStringSet(oracle.requestedScopes, "oracle requested scopes");
  assert.ok(oracle.requestedScopes.includes(oracle.rootScope));
  assertStringSet(oracle.unresolvedScopeSentinels, "oracle unresolved scope sentinels");
  assert.ok(Array.isArray(grammars.vize) && grammars.vize.length >= 1 && grammars.vize.length <= 2);
  const roots: string[] = [];
  for (const grammar of grammars.vize) {
    assertRecord(grammar, "Vize grammar evidence");
    assert.deepEqual(Object.keys(grammar), [
      "configuredGrammarSha256",
      "dependencyVersions",
      "requestedScopes",
      "rootScope",
    ]);
    assert.match(grammar.configuredGrammarSha256, /^[0-9a-f]{64}$/);
    assert.deepEqual(grammar.dependencyVersions, textmateDependencyVersions);
    assertStringSet(grammar.requestedScopes, "Vize requested scopes");
    assert.ok(grammar.requestedScopes.includes(grammar.rootScope));
    roots.push(grammar.rootScope);
  }
  assert.deepEqual(roots, roots.length === 1 ? ["source.vue"] : ["source.vue", "source.art-vue"]);
}

function validateRecord(value: unknown, collection: string, projectId: string): void {
  assert.ok(value != null && typeof value === "object" && !Array.isArray(value));
  const record = value as Record<string, any>;
  const documented = collection === "documentedDivergences";
  assert.deepEqual(
    Object.keys(record).sort(),
    [
      "category",
      "endColumn",
      "file",
      ...(documented ? ["issue"] : []),
      "kind",
      "line",
      ...(documented ? ["project", "reason"] : []),
      "startColumn",
    ].sort(),
  );
  assertNormalizedPath(record.file, "artifact path");
  assert.ok(
    semanticCategories.includes(record.category),
    `unknown artifact semantic category ${String(record.category)}`,
  );
  const expectedKind = collection === "falsePositives" ? "false-positive" : "false-negative";
  if (!documented) assert.equal(record.kind, expectedKind);
  else {
    assert.ok(record.kind === "false-positive" || record.kind === "false-negative");
    assert.equal(record.project, projectId);
    assert.ok(Number.isSafeInteger(record.issue) && record.issue > 0);
    assert.ok(typeof record.reason === "string" && record.reason.trim().length >= 40);
  }
  assert.ok(Number.isSafeInteger(record.line) && record.line > 0);
  assert.ok(
    Number.isSafeInteger(record.startColumn) &&
      Number.isSafeInteger(record.endColumn) &&
      record.startColumn > 0 &&
      record.endColumn > record.startColumn,
  );
}

function sum(projects: ProjectArtifact[], key: keyof ProjectArtifact): number {
  return projects.reduce((total, project) => total + (project[key] as unknown[]).length, 0);
}

function summarize(projects: ProjectArtifact[]): DivergenceSummary {
  return {
    documentedDivergenceCount: sum(projects, "documentedDivergences"),
    falseNegativeCount: sum(projects, "falseNegatives"),
    falsePositiveCount: sum(projects, "falsePositives"),
    fileCount: projects.reduce((total, project) => total + project.fileCount, 0),
    projectCount: projects.length,
    semanticSpanCount: projects.reduce((total, project) => total + project.semanticSpanCount, 0),
  };
}

function assertRecord(value: unknown, label: string): void {
  assert.ok(value != null && typeof value === "object" && !Array.isArray(value), label);
}

function assertStringSet(value: unknown, label: string): void {
  assert.ok(
    Array.isArray(value) && value.every((item) => typeof item === "string" && item.length > 0),
    label,
  );
  const strings = value as string[];
  assert.deepEqual(
    strings,
    [...new Set(strings)].sort(byteOrder),
    `${label} must be sorted and unique`,
  );
}
