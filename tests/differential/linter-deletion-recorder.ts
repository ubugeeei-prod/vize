import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  type BuildReceipt,
  type LintIndex,
  type LintProjectReport,
  type LinterDeletionArtifact,
  type RegistryProject,
  commitPattern,
  sha256,
} from "./linter-deletion-schema.ts";
import {
  type EvidenceFile,
  type FixtureToolMatrix,
  type MatrixRun,
  type RawLinterRun,
} from "./linter-deletion-matrix.ts";
import { assessLinterDeletionShard, linterProjects } from "./linter-deletion-verifier.ts";

function readEvidence<T>(filename: string): EvidenceFile<T> {
  const bytes = fs.readFileSync(filename);
  return { value: JSON.parse(bytes.toString("utf8")) as T, sha256: sha256(bytes) };
}

function exactOutputPath(
  root: string,
  reportDir: string,
  projectId: string,
  run: MatrixRun,
): string {
  const candidate = path.resolve(root, run.outputPath),
    expected = path.resolve(reportDir, `${projectId}-linter.json`),
    relative = path.relative(reportDir, candidate);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  assert.equal(candidate, expected, `${projectId}: matrix linter output path drifted`);
  return candidate;
}

export function recordLinterDeletionShard(root: string, reportDir: string): LinterDeletionArtifact {
  const sourceRevision = process.env.GITHUB_SHA ?? "";
  assert.match(sourceRevision, commitPattern, "GITHUB_SHA must be an exact commit");
  const shardIndex = Number(process.env.FIXTURE_SHARD_INDEX),
    shardCount = Number(process.env.FIXTURE_SHARD_COUNT),
    registryPath = path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"),
    registryFile = readEvidence<{ projects: RegistryProject[] }>(registryPath),
    selected = linterProjects(registryFile.value.projects, shardIndex, shardCount),
    matrix = readEvidence<FixtureToolMatrix>(path.join(reportDir, "summary.json")),
    matrixRuns = new Map(
      matrix.value.projects.map((project) => [
        project.id,
        project.runs.find((run) => run.tool === "linter") as MatrixRun,
      ]),
    ),
    rawLinterRuns = new Map(
      selected.map((project) => {
        const run = matrixRuns.get(project.id);
        assert(run, `${project.id}: matrix linter run is missing`);
        return [
          project.id,
          readEvidence<RawLinterRun>(exactOutputPath(root, reportDir, project.id, run)),
        ];
      }),
    ),
    projectArtifacts = new Map(
      selected.map((project) => [
        project.id,
        readEvidence<LintProjectReport>(path.join(reportDir, `${project.id}-lint-divergence.json`)),
      ]),
    );
  const artifact = assessLinterDeletionShard({
    sourceRevision,
    registrySha256: registryFile.sha256,
    shardIndex,
    shardCount,
    registryProjects: registryFile.value.projects,
    matrix,
    rawLinterRuns,
    index: readEvidence<LintIndex>(path.join(reportDir, "lint-divergence-summary.json")),
    projectArtifacts,
    ruleMap: readEvidence<unknown>(
      path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json"),
    ),
    divergenceLedgerSha256: sha256(
      fs.readFileSync(path.join(root, "tests/_fixtures/patina-lint-documented-divergences.json")),
    ),
    buildReceipt: readEvidence<BuildReceipt>(
      path.join(root, "target/ci/vize.differential-build.json"),
    ).value,
  });
  const output = path.join(reportDir, "linter-deletion-readiness.json");
  fs.writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  process.stdout.write(
    `linter deletion evidence: ${artifact.status}, ${artifact.totals.projectCount} project(s), ` +
      `${artifact.totals.comparedFileCount} file(s), ${artifact.blockers.length} blocker(s)\n`,
  );
  return artifact;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
    reportDir = path.resolve(process.argv[2] ?? "");
  assert(process.argv[2], "usage: linter-deletion-recorder.ts REPORT_DIR");
  recordLinterDeletionShard(root, reportDir);
}
