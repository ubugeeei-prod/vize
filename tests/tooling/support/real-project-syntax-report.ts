import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { renderDivergenceMarkdown, type DivergenceArtifact } from "./syntax-divergence-artifact.ts";
import type { TextMateGrammarEvidence } from "./vue-textmate.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export type ProjectEvidence = {
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

export type ShardSelection = {
  count: number | null;
  index: number | null;
};

export function writeStructuralEvidence(
  shard: ShardSelection,
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
    shard,
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

export function writeDivergenceEvidence(
  artifact: DivergenceArtifact,
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

export function resolveCommitSha(environment: NodeJS.ProcessEnv): string {
  const value = environment.GITHUB_SHA;
  if (value != null) return requireCommitSha(value, "GITHUB_SHA");
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, "git rev-parse HEAD must succeed");
  return requireCommitSha(result.stdout.trim(), "git rev-parse HEAD");
}

function outputDirectory(environment: NodeJS.ProcessEnv): string | null {
  const value = environment.FIXTURE_REPORT_DIR;
  if (value == null || value.length === 0) return null;
  const outputDir = path.resolve(root, value);
  fs.mkdirSync(outputDir, { recursive: true });
  return outputDir;
}

function requireCommitSha(value: string, source: string): string {
  assert.match(value, /^[0-9a-f]{40}$/, `${source} must be a full lowercase commit SHA`);
  return value;
}
