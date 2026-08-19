import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { typecheckPerformanceProjectIds } from "../fixtures/typecheck-performance-shard.mjs";
import { readJsonEntry, readTextEntry } from "./release-preflight-artifact-entries.mjs";
import {
  assertReleaseTypecheckCoverage,
  assertReleaseTypecheckShardArtifacts,
} from "./release-preflight-typecheck-evidence.mjs";

export const requiredRealProjectMatrixShardCount = 22;
export const realProjectMatrixWorkflowName = "Real Project Matrix";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const defaultRegistry = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

// Release evidence for the full corpus is mandatory: a selection that omits the
// workflow must fail the gate instead of silently skipping all shard artifacts.
export function requireRealProjectMatrixRun(selectedRuns) {
  const run = selectedRuns?.get?.(realProjectMatrixWorkflowName);
  if (run == null) {
    throw new Error(
      `${realProjectMatrixWorkflowName} release evidence is required; no exact-SHA run was selected`,
    );
  }
  return run;
}

export async function assertRealProjectMatrixReleaseArtifacts({
  run,
  artifacts,
  readArtifactEntries,
  registry = readDefaultTypecheckRegistry(),
  enforceParity,
}) {
  if (typeof readArtifactEntries !== "function") {
    throw new Error("Real Project Matrix artifact reader is required");
  }
  const expectedTypecheckProjects = new Set(typecheckPerformanceProjectIds(registry));
  const observedTypecheckProjects = new Map();
  const expectedNames = Array.from(
    { length: requiredRealProjectMatrixShardCount },
    (_, shard) => `real-project-matrix-${shard}`,
  );
  for (const artifactName of expectedNames) {
    const matches = artifacts.filter((artifact) => artifact.name === artifactName);
    if (matches.length !== 1) {
      throw new Error(
        `Real Project Matrix release evidence must contain exactly one ${artifactName} artifact; found ${matches.length}`,
      );
    }
    const artifact = matches[0];
    assertArtifactBoundToRun(run, artifact);
    const entries = await readArtifactEntries(artifact);
    assertRealProjectShardArtifact({
      run,
      artifactName,
      shard: Number(artifactName.slice("real-project-matrix-".length)),
      entries,
      expectedTypecheckProjects,
      observedTypecheckProjects,
      enforceParity,
    });
  }
  assertReleaseTypecheckCoverage(expectedTypecheckProjects, observedTypecheckProjects);
}

function assertRealProjectShardArtifact({
  run,
  artifactName,
  shard,
  entries,
  expectedTypecheckProjects,
  observedTypecheckProjects,
  enforceParity,
}) {
  const summary = readJsonEntry(entries, "summary.json", artifactName);
  if (
    summary.schema !== "vize.fixtureToolMatrixReport" ||
    summary.version !== 3 ||
    summary.evidence?.commitSha !== run.head_sha ||
    summary.command?.shardIndex !== shard ||
    summary.command?.shardCount !== requiredRealProjectMatrixShardCount
  ) {
    throw new Error(`${artifactName} summary is not exact release evidence for ${run.head_sha}`);
  }
  const selectedFixtures = readTextEntry(entries, "selected-fixtures.txt", artifactName)
    .split(/\r?\n/)
    .filter(Boolean);
  if (selectedFixtures.length === 0) {
    throw new Error(`${artifactName} selected no authored fixture corpus`);
  }

  const surface = readJsonEntry(entries, "surface-verdict.json", artifactName);
  if (surface.status !== "success") {
    throw new Error(`${artifactName} surface verdict is ${String(surface.status)}`);
  }
  assertReleaseLintDivergenceSummary({
    artifactName,
    run,
    summary: readJsonEntry(entries, "lint-divergence-summary.json", artifactName),
  });

  assertReleaseTypecheckShardArtifacts({
    artifactName,
    run,
    entries,
    expectedTypecheckProjects,
    observedTypecheckProjects,
    enforceParity,
  });
}

function assertReleaseLintDivergenceSummary({ artifactName, run, summary }) {
  if (
    summary.schema !== "vize.fixtureLintDivergenceIndex" ||
    summary.version !== 1 ||
    summary.evidence?.commitSha !== run.head_sha ||
    !Number.isSafeInteger(summary.projectCount) ||
    summary.projectCount <= 0 ||
    !Array.isArray(summary.projects) ||
    summary.projects.length !== summary.projectCount
  ) {
    throw new Error(`${artifactName} lint divergence summary is not exact release evidence`);
  }
}

function readDefaultTypecheckRegistry() {
  return JSON.parse(readFileSync(defaultRegistry, "utf8"));
}

function assertArtifactBoundToRun(run, artifact) {
  if (artifact.expired === true) {
    throw new Error(`Real Project Matrix artifact ${String(artifact.name)} has expired`);
  }
  const source = artifact.workflow_run;
  if (
    source == null ||
    Number(source.id) !== Number(run.id) ||
    source.head_sha !== run.head_sha ||
    source.head_branch !== run.head_branch
  ) {
    throw new Error(
      `Real Project Matrix artifact ${String(artifact.name)} is not bound to run ${String(run.id)}`,
    );
  }
}
