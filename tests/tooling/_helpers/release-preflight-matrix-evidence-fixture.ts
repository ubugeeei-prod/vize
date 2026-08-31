import { createHash } from "node:crypto";

import { requiredRealProjectMatrixShardCount } from "../../../legacy-tools/github/release-preflight-matrix-evidence.mjs";
import { releaseSha, successfulReleaseRun } from "../support/release-preflight.ts";

export function realProjectArtifacts(run: ReturnType<typeof successfulReleaseRun>) {
  return Array.from({ length: requiredRealProjectMatrixShardCount }, (_, shard) => ({
    id: 1_000 + shard,
    name: `real-project-matrix-${shard}`,
    expired: false,
    archive_download_url: `https://example.test/artifacts/${shard}.zip`,
    workflow_run: {
      id: run.id,
      head_branch: run.head_branch,
      head_sha: run.head_sha,
    },
  }));
}

export function typecheckRegistry(projects = defaultTypecheckProjects()) {
  return {
    projects: projects.map((id) => ({
      id,
      typecheckPerformance: { enabled: true },
    })),
  };
}

export function defaultTypecheckProjects() {
  return Array.from(
    { length: requiredRealProjectMatrixShardCount },
    (_, shard) => `fixture-${shard}`,
  );
}

export function shardEntries(
  shard: number,
  options: { typecheckProject?: string | null } = {},
): Record<string, string> {
  const projectId =
    options.typecheckProject === undefined ? `fixture-${shard}` : options.typecheckProject;
  const entries: Record<string, string> = {
    "selected-fixtures.txt": "tests/_fixtures/_git/fixture\n",
    "summary.json": json({
      schema: "vize.fixtureToolMatrixReport",
      version: 3,
      evidence: { commitSha: releaseSha },
      command: { shardIndex: shard, shardCount: requiredRealProjectMatrixShardCount },
    }),
    "surface-verdict.json": json({ status: "success" }),
    "lint-divergence-summary.json": json({
      schema: "vize.fixtureLintDivergenceIndex",
      version: 1,
      evidence: { commitSha: releaseSha },
      projectCount: 1,
      projects: [{ project: "fixture" }],
      budget: { status: "failure", passed: false },
    }),
  };
  if (projectId == null) return entries;
  const dependency = {
    schema: "vize.fixtureTypecheckDependencyInstall",
    version: 2,
    project: projectId,
    revision: "b".repeat(40),
    evidence: { commitSha: releaseSha },
    packageManager: { name: "pnpm", version: "10.0.0" },
    lockfile: { path: "pnpm-lock.yaml", sizeBytes: 18, sha256: "1".repeat(64) },
    install: {
      command: ["pnpm", "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
      durationMs: 1,
      exitCode: 0,
      stdoutSha256: "2".repeat(64),
      stderrSha256: "3".repeat(64),
    },
    baselinePrepare: null,
  };
  const dependencyText = json(dependency);
  entries[`${projectId}-typecheck-dependencies.json`] = dependencyText;
  entries[`${projectId}-typecheck-divergence.json`] = json({
    schema: "vize.fixtureTypecheckDivergenceRun",
    version: 6,
    project: projectId,
    revision: "b".repeat(40),
    evidence: { commitSha: releaseSha },
    enforcement: { budgetMode: "enforce" },
    preparation: {
      schema: "vize.fixtureTypecheckPreparationEvidence",
      version: 1,
      payloadSha256: sha256(dependencyText),
    },
    baseline: {
      coverage: {
        verdict: "usable",
        vizeVueFileCount: 1,
        baselineVueFileCount: 1,
        sharedVueFileCount: 1,
        vizeVueFilesSha256: "4".repeat(64),
        baselineVueFilesSha256: "4".repeat(64),
        missingVueFiles: [],
        unexpectedVueFiles: [],
      },
    },
    mutationOracle: {
      schema: "vize.fixtureTypecheckSeededMutationOracle",
      version: 1,
      verdict: "passed",
      passed: true,
      file: "src/App.vue",
      span: { line: 3, column: 1 },
      cleanExpectedDiagnosticPresent: false,
      expectedDiagnosticMatched: true,
      repairedExpectedDiagnosticPresent: false,
      states: [
        mutationState("clean", "5".repeat(64), 0, 0, 0),
        mutationState("broken", "6".repeat(64), 1, 0, 0),
        mutationState("repaired", "5".repeat(64), 0, 0, 0),
      ],
    },
    budget: { passed: true, verdict: "passed" },
    divergence: {
      summary: {
        falsePositiveCount: 0,
        falseNegativeCount: 0,
      },
    },
  });
  return entries;
}

export function mutateDivergence(entries: Record<string, string>, mutate: (artifact: any) => void) {
  const name = Object.keys(entries).find((entry) => entry.endsWith("-typecheck-divergence.json"));
  if (name == null) throw new Error("No typecheck divergence artifact in fixture entries");
  const artifact = JSON.parse(entries[name]);
  mutate(artifact);
  entries[name] = json(artifact);
}

export function json(value: unknown) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function mutationState(
  name: string,
  sourceSha256: string,
  sharedCount: number,
  falsePositiveCount: number,
  falseNegativeCount: number,
) {
  const summary = {
    vizeDiagnosticCount: sharedCount,
    baselineDiagnosticCount: sharedCount,
    sharedCount,
    messageMismatchCount: 0,
    documentedDifferenceCount: 0,
    falsePositiveCount,
    falseNegativeCount,
  };
  return {
    name,
    sourceSha256,
    ...summary,
    observed: summary,
    vize: mutationRun("vize", falsePositiveCount > 0 ? 1 : 0),
    baseline: mutationRun("vue-tsc", falseNegativeCount > 0 ? 2 : 0),
  };
}

function mutationRun(command: string, exitCode: number) {
  return { command, exitCode, stdoutSha256: "8".repeat(64), stderrSha256: "9".repeat(64) };
}

function sha256(value: string) {
  return createHash("sha256").update(value).digest("hex");
}
