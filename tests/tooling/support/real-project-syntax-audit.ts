import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { collectVueInputPaths } from "../../../tools/fixtures/tool-matrix-inputs.mjs";
import { loadPinnedShikiVueOracle } from "./shiki-vue-oracle.ts";
import {
  createDivergenceArtifact,
  renderDivergenceMarkdown,
  type ProjectArtifact,
} from "./syntax-divergence-artifact.ts";
import {
  applyDocumentedDivergences,
  compareSemanticSpans,
  validateLedger,
  type DivergenceRecord,
} from "./syntax-semantic-comparison.ts";
import { sha256, tokenizeSemanticSource } from "./syntax-semantic-divergence.ts";
import { createSyntaxAuditDeadline } from "./syntax-audit-deadline.ts";
import {
  loadVueTextMateGrammar,
  type TextMateGrammar,
  type TextMateGrammarEvidence,
} from "./vue-textmate.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const ledgerPath = path.join(
  root,
  "tests",
  "_fixtures",
  "syntax-highlighter-documented-divergences.json",
);

type FixtureProject = {
  coverage: string[];
  expectedVueFileCount: number | null;
  fixturePath: string;
  id: string;
  revision: string;
  vueGlobs: string[];
};

type ProjectEvidence = {
  characterCount: number;
  durationMs: number;
  failure?: string;
  fileCount: number;
  id: string;
  inputSha256: string;
  lineCount: number;
  revision: string;
  status: "failed" | "ok";
  tokenCount: number;
  tokenSha256: string;
};

export async function runRealProjectSyntaxAudit(environment = process.env, now = Date.now) {
  const checkDeadline = createSyntaxAuditDeadline(environment, now);
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as {
    projects: FixtureProject[];
    requiredToolCoverage: string[];
  };
  assert.ok(registry.requiredToolCoverage.includes("syntax-highlighter"));
  const selection = selectProjects(registry.projects, environment);
  if (!selection.enforced && selection.projects.length === 0) return { skipped: true };
  const rawLedger = fs.readFileSync(ledgerPath, "utf8");
  const ledger = JSON.parse(rawLedger) as unknown;
  validateLedger(ledger, new Set(registry.projects.map((project) => project.id)));
  const vizeVue = await loadVueTextMateGrammar();
  const oracle = await loadPinnedShikiVueOracle();
  let vizeArt: Awaited<ReturnType<typeof loadVueTextMateGrammar>> | null = null;
  const evidence: ProjectEvidence[] = [];
  const projectArtifacts: ProjectArtifact[] = [];

  try {
    checkDeadline();
    for (const project of selection.projects) {
      const fixtureDir = path.resolve(root, project.fixturePath);
      const startedAt = Date.now();
      const structural = emptyEvidence(project);
      try {
        assert.ok(project.coverage.includes("syntax-highlighter"));
        assert.ok(isHydrated(fixtureDir), `${project.id} fixture is not hydrated`);
        const files = collectVueInputPaths(fixtureDir, project.vueGlobs);
        assertFixtureFileCount(project, files);
        structural.fileCount = files.length;
        const inputDigest = createHash("sha256");
        const tokenDigest = createHash("sha256");
        const comparisonDigest = createHash("sha256");
        const falsePositives: DivergenceRecord[] = [];
        const falseNegatives: DivergenceRecord[] = [];
        let semanticSpanCount = 0;

        for (const file of files) {
          checkDeadline();
          const source = fs.readFileSync(path.join(fixtureDir, file), "utf8");
          const isArtVue = file.endsWith(".art.vue");
          if (isArtVue && vizeArt == null) {
            vizeArt = await loadVueTextMateGrammar("source.art-vue");
            checkDeadline();
          }
          const vizeGrammar = isArtVue ? (vizeArt?.grammar as TextMateGrammar) : vizeVue.grammar;
          const vizeRoot = isArtVue ? "source.art-vue" : "source.vue";
          const label = `${project.id}/${file}`;
          const vize = tokenizeSemanticSource(
            vizeGrammar,
            source,
            vizeRoot,
            `vize:${label}`,
            checkDeadline,
          );
          const baseline = tokenizeSemanticSource(
            oracle.grammar,
            source,
            oracle.rootScope,
            `oracle:${label}`,
            checkDeadline,
          );
          assert.equal(vize.lineCount, baseline.lineCount, `${label}: line count drift`);
          const comparison = compareSemanticSpans(
            file,
            source,
            vize.semanticSpans,
            baseline.semanticSpans,
          );
          checkDeadline();
          falsePositives.push(...comparison.falsePositives);
          falseNegatives.push(...comparison.falseNegatives);
          semanticSpanCount +=
            comparison.shared.length +
            comparison.falsePositives.length +
            comparison.falseNegatives.length;
          structural.characterCount += source.length;
          structural.lineCount += vize.lineCount;
          structural.tokenCount += vize.tokenCount;
          inputDigest.update(`${JSON.stringify([file, source])}\n`);
          tokenDigest.update(`${JSON.stringify([file, vize.tokenSha256])}\n`);
          comparisonDigest.update(`${JSON.stringify([file, comparison.sha256])}\n`);
        }
        structural.inputSha256 = inputDigest.digest("hex");
        structural.tokenSha256 = tokenDigest.digest("hex");
        const classified = applyDocumentedDivergences(
          project.id,
          { falseNegatives, falsePositives, sha256: "", shared: [] },
          ledger,
        );
        projectArtifacts.push({
          id: project.id,
          revision: project.revision,
          fileCount: files.length,
          inputSha256: structural.inputSha256,
          semanticSpanCount,
          comparisonSha256: comparisonDigest.digest("hex"),
          falsePositives: classified.falsePositives,
          falseNegatives: classified.falseNegatives,
          documentedDivergences: classified.documented,
        });
        checkDeadline();
        structural.status = "ok";
      } catch (error) {
        structural.failure = errorMessage(error);
      }
      structural.durationMs = Date.now() - startedAt;
      evidence.push(structural);
    }
  } finally {
    vizeVue.registry.dispose();
    vizeArt?.registry.dispose();
    oracle.registry.dispose();
  }

  writeStructuralEvidence(
    selection,
    evidence,
    [vizeVue.getEvidence(), vizeArt?.getEvidence()].filter(
      (item): item is TextMateGrammarEvidence => item != null,
    ),
    environment,
  );
  const failures = evidence.filter((project) => project.status === "failed");
  assert.deepEqual(
    failures.map((project) => `${project.id}: ${project.failure}`),
    [],
  );
  const commitSha = resolveCommitSha(environment);
  checkDeadline();
  const artifact = createDivergenceArtifact({
    commitSha,
    grammars: {
      oracle: oracle.getEvidence(),
      vize: [vizeVue.getEvidence(), vizeArt?.getEvidence()].filter(Boolean),
    },
    ledger: {
      path: path.relative(root, ledgerPath).replaceAll("\\", "/"),
      sha256: sha256(rawLedger),
    },
    projects: projectArtifacts,
    shard: { count: selection.shardCount, index: selection.shardIndex },
  });
  writeDivergenceEvidence(artifact, environment);
  process.stderr.write(
    `syntax oracle: ${artifact.summary.projectCount} project(s), ${artifact.summary.fileCount} file(s), ` +
      `${artifact.summary.falsePositiveCount} FP, ${artifact.summary.falseNegativeCount} FN\n`,
  );
  return { artifact, evidence, skipped: false };
}

function selectProjects(projects: FixtureProject[], environment: NodeJS.ProcessEnv) {
  const shardIndex = environment.FIXTURE_SHARD_INDEX;
  const shardCount = environment.FIXTURE_SHARD_COUNT;
  if ((shardIndex == null) !== (shardCount == null)) {
    throw new Error("FIXTURE_SHARD_INDEX and FIXTURE_SHARD_COUNT must be set together");
  }
  if (shardIndex != null && shardCount != null) {
    const index = nonNegativeInteger(shardIndex, "FIXTURE_SHARD_INDEX");
    const count = positiveInteger(shardCount, "FIXTURE_SHARD_COUNT");
    assert.ok(index < count, "FIXTURE_SHARD_INDEX must be less than FIXTURE_SHARD_COUNT");
    const selected = projects.filter((_, projectIndex) => projectIndex % count === index);
    assert.ok(selected.length > 0, `fixture shard ${index}/${count} selected no projects`);
    return { enforced: true, projects: selected, shardCount: count, shardIndex: index };
  }
  return {
    enforced: false,
    projects: projects.filter((project) => isHydrated(path.resolve(root, project.fixturePath))),
    shardCount: null,
    shardIndex: null,
  };
}

function writeStructuralEvidence(
  selection: ReturnType<typeof selectProjects>,
  projects: ProjectEvidence[],
  grammars: TextMateGrammarEvidence[],
  environment: NodeJS.ProcessEnv,
): void {
  const outputDir = outputDirectory(environment);
  if (outputDir == null) return;
  const artifact = {
    schema: "vize.fixtureSyntaxHighlighterReport",
    version: 1,
    generatedAt: new Date().toISOString(),
    commitSha: resolveCommitSha(environment),
    runtime: { name: "node", version: process.versions.node },
    machine: {
      arch: process.arch,
      logicalCpuCount: os.cpus().length,
      platform: process.platform,
      totalMemoryBytes: os.totalmem(),
    },
    shard: { count: selection.shardCount, index: selection.shardIndex },
    grammars,
    summary: {
      failedProjectCount: projects.filter((project) => project.status === "failed").length,
      fileCount: projects.reduce((sum, project) => sum + project.fileCount, 0),
      lineCount: projects.reduce((sum, project) => sum + project.lineCount, 0),
      projectCount: projects.length,
      tokenCount: projects.reduce((sum, project) => sum + project.tokenCount, 0),
    },
    projects,
  };
  fs.writeFileSync(
    path.join(outputDir, "syntax-highlighter-summary.json"),
    `${JSON.stringify(artifact, null, 2)}\n`,
  );
}

function writeDivergenceEvidence(
  artifact: ReturnType<typeof createDivergenceArtifact>,
  environment: NodeJS.ProcessEnv,
): void {
  const outputDir = outputDirectory(environment);
  if (outputDir == null) return;
  fs.writeFileSync(
    path.join(outputDir, "syntax-highlighter-divergence.json"),
    `${JSON.stringify(artifact, null, 2)}\n`,
  );
  fs.writeFileSync(
    path.join(outputDir, "syntax-highlighter-divergence.md"),
    renderDivergenceMarkdown(artifact),
  );
}

function outputDirectory(environment: NodeJS.ProcessEnv): string | null {
  const value = environment.FIXTURE_REPORT_DIR;
  if (value == null || value.length === 0) return null;
  const outputDir = path.resolve(root, value);
  fs.mkdirSync(outputDir, { recursive: true });
  return outputDir;
}

function emptyEvidence(project: FixtureProject): ProjectEvidence {
  return {
    characterCount: 0,
    durationMs: 0,
    fileCount: 0,
    id: project.id,
    inputSha256: sha256(""),
    lineCount: 0,
    revision: project.revision,
    status: "failed",
    tokenCount: 0,
    tokenSha256: sha256(""),
  };
}

function assertFixtureFileCount(project: FixtureProject, files: string[]): void {
  if (project.expectedVueFileCount != null)
    assert.equal(files.length, project.expectedVueFileCount);
  if (project.expectedVueFileCount !== 0)
    assert.ok(files.length > 0, `${project.id} matched no files`);
}

function isHydrated(directory: string): boolean {
  return fs.existsSync(directory) && fs.readdirSync(directory).length > 0;
}

function resolveCommitSha(environment: NodeJS.ProcessEnv): string {
  const value = environment.GITHUB_SHA;
  if (value != null) return requireCommitSha(value, "GITHUB_SHA");
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, "git rev-parse HEAD must succeed");
  return requireCommitSha(result.stdout.trim(), "git rev-parse HEAD");
}

function requireCommitSha(value: string, source: string): string {
  assert.match(value, /^[0-9a-f]{40}$/, `${source} must be a full lowercase commit SHA`);
  return value;
}

function positiveInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) throw new Error(`${name} must be positive`);
  return parsed;
}

function nonNegativeInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) throw new Error(`${name} must be non-negative`);
  return parsed;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
