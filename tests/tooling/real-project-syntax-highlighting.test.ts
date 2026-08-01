import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { collectVueInputPaths } from "../../tools/fixtures/tool-matrix-inputs.mjs";
import {
  auditTextMateSource,
  loadVueTextMateGrammar,
  type TextMateGrammar,
  type TextMateGrammarEvidence,
} from "./support/vue-textmate.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

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
  lineCount: number;
  revision: string;
  inputSha256: string;
  status: "failed" | "ok";
  tokenSha256: string;
  tokenCount: number;
};

test("shipped TextMate grammar tokenizes every selected real-project Vue file", async () => {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as {
    projects: FixtureProject[];
    requiredToolCoverage: string[];
  };
  assert.ok(
    registry.requiredToolCoverage.includes("syntax-highlighter"),
    "registry must require syntax-highlighter coverage",
  );

  const selection = selectProjects(registry.projects);
  if (!selection.enforced && selection.projects.length === 0) return;
  const vue = await loadVueTextMateGrammar();
  let artVue: Awaited<ReturnType<typeof loadVueTextMateGrammar>> | null = null;
  const evidence: ProjectEvidence[] = [];

  try {
    for (const project of selection.projects) {
      const fixtureDir = path.resolve(root, project.fixturePath);
      const startedAt = Date.now();
      let fileCount = 0;
      let lineCount = 0;
      let tokenCount = 0;
      let characterCount = 0;
      const inputDigest = createHash("sha256");
      const tokenDigest = createHash("sha256");

      try {
        assert.ok(
          project.coverage.includes("syntax-highlighter"),
          `${project.id} does not declare syntax-highlighter coverage`,
        );
        assert.ok(isHydrated(fixtureDir), `${project.id} fixture is not hydrated`);
        const files = collectVueInputPaths(fixtureDir, project.vueGlobs);
        assertFixtureFileCount(project, files);
        fileCount = files.length;

        for (const file of files) {
          const source = fs.readFileSync(path.join(fixtureDir, file), "utf8");
          const isArtVue = file.endsWith(".art.vue");
          if (isArtVue && artVue == null) {
            artVue = await loadVueTextMateGrammar("source.art-vue");
          }
          const grammar = isArtVue ? (artVue?.grammar as TextMateGrammar) : vue.grammar;
          const result = auditTextMateSource(
            grammar,
            source,
            isArtVue ? "source.art-vue" : "source.vue",
            `${project.id}/${file}`,
          );
          lineCount += result.lineCount;
          tokenCount += result.tokenCount;
          characterCount += source.length;
          inputDigest.update(`${JSON.stringify([file, source])}\n`);
          tokenDigest.update(`${JSON.stringify([file, result.sha256])}\n`);
        }
        evidence.push({
          characterCount,
          durationMs: Date.now() - startedAt,
          fileCount,
          id: project.id,
          lineCount,
          revision: project.revision,
          inputSha256: inputDigest.digest("hex"),
          status: "ok",
          tokenSha256: tokenDigest.digest("hex"),
          tokenCount,
        });
      } catch (error) {
        evidence.push({
          characterCount,
          durationMs: Date.now() - startedAt,
          failure: errorMessage(error),
          fileCount,
          id: project.id,
          lineCount,
          revision: project.revision,
          inputSha256: inputDigest.digest("hex"),
          status: "failed",
          tokenSha256: tokenDigest.digest("hex"),
          tokenCount,
        });
      }
    }
  } finally {
    vue.registry.dispose();
    artVue?.registry.dispose();
  }

  writeEvidence(
    selection,
    evidence,
    [vue.getEvidence(), artVue?.getEvidence()].filter(
      (item): item is TextMateGrammarEvidence => item != null,
    ),
  );
  const failures = evidence.filter((project) => project.status === "failed");
  const totals = evidence.reduce(
    (sum, project) => ({
      files: sum.files + project.fileCount,
      lines: sum.lines + project.lineCount,
      tokens: sum.tokens + project.tokenCount,
    }),
    { files: 0, lines: 0, tokens: 0 },
  );
  process.stderr.write(
    `syntax highlighter: ${evidence.length} project(s), ${totals.files} file(s), ` +
      `${totals.lines} line(s), ${totals.tokens} token(s), ${failures.length} failure(s)\n`,
  );
  assert.deepEqual(
    failures.map((project) => `${project.id}: ${project.failure}`),
    [],
  );
});

test("real-project TextMate audit exercises the shipped grammar and fail-closed spans", async () => {
  await assert.rejects(
    () => loadVueTextMateGrammar("source.vize-missing-test-grammar"),
    /unresolved TextMate grammar scope: source\.vize-missing-test-grammar/,
  );
  const { grammar, registry } = await loadVueTextMateGrammar();
  try {
    const source = `<script setup lang="ts">\nconst label = 'ready'\n</script>\n<template>{{ label }}</template>\n`;
    const result = auditTextMateSource(grammar, source, "source.vue", "synthetic/App.vue");
    assert.equal(result.lineCount, 5);
    assert.ok(result.tokenCount > 5);
  } finally {
    registry.dispose();
  }
  const artVue = await loadVueTextMateGrammar("source.art-vue");
  try {
    const result = auditTextMateSource(
      artVue.grammar,
      '<art title="Button"><variant name="primary"><button /></variant></art>\n',
      "source.art-vue",
      "synthetic/Button.art.vue",
    );
    assert.ok(result.tokenCount > 5);
  } finally {
    artVue.registry.dispose();
  }

  const gapGrammar = {
    tokenizeLine() {
      return {
        ruleStack: null,
        tokens: [{ startIndex: 1, endIndex: 2, scopes: ["source.vue", "meta.tag.vue"] }],
      };
    },
  };
  assert.throws(
    () => auditTextMateSource(gapGrammar, "x", "source.vue", "gap.vue"),
    /not a contiguous positive span/,
  );
  const slowGrammar = {
    tokenizeLine() {
      return {
        ruleStack: null,
        stoppedEarly: true,
        tokens: [{ startIndex: 0, endIndex: 1, scopes: ["source.vue"] }],
      };
    },
  };
  assert.throws(
    () => auditTextMateSource(slowGrammar, "x", "source.vue", "slow.vue"),
    /exceeded 250ms/,
  );
  const oneTokenGrammar = (scope: string) => ({
    tokenizeLine() {
      return {
        ruleStack: null,
        tokens: [{ startIndex: 0, endIndex: 1, scopes: ["source.vue", scope] }],
      };
    },
  });
  const first = auditTextMateSource(oneTokenGrammar("meta.first"), "x", "source.vue", "first.vue");
  const second = auditTextMateSource(
    oneTokenGrammar("meta.second"),
    "x",
    "source.vue",
    "second.vue",
  );
  assert.notEqual(first.sha256, second.sha256, "token digest must include scope identity");
});

function selectProjects(projects: FixtureProject[]): {
  enforced: boolean;
  projects: FixtureProject[];
  shardCount: number | null;
  shardIndex: number | null;
} {
  const shardIndex = process.env.FIXTURE_SHARD_INDEX;
  const shardCount = process.env.FIXTURE_SHARD_COUNT;
  if ((shardIndex == null) !== (shardCount == null)) {
    throw new Error("FIXTURE_SHARD_INDEX and FIXTURE_SHARD_COUNT must be set together");
  }
  if (shardIndex != null && shardCount != null) {
    const index = nonNegativeInteger(shardIndex, "FIXTURE_SHARD_INDEX");
    const count = positiveInteger(shardCount, "FIXTURE_SHARD_COUNT");
    assert.ok(index < count, "FIXTURE_SHARD_INDEX must be less than FIXTURE_SHARD_COUNT");
    const selected = projects.filter((_, projectIndex) => projectIndex % count === index);
    assert.ok(selected.length > 0, `fixture shard ${index}/${count} selected no projects`);
    return {
      enforced: true,
      projects: selected,
      shardCount: count,
      shardIndex: index,
    };
  }
  return {
    enforced: false,
    projects: projects.filter((project) => isHydrated(path.resolve(root, project.fixturePath))),
    shardCount: null,
    shardIndex: null,
  };
}

function assertFixtureFileCount(project: FixtureProject, files: string[]): void {
  if (project.expectedVueFileCount != null) {
    assert.equal(
      files.length,
      project.expectedVueFileCount,
      `${project.id} matched ${files.length} Vue files, expected ${project.expectedVueFileCount}`,
    );
  }
  if (project.expectedVueFileCount !== 0) {
    assert.ok(files.length > 0, `${project.id} matched no Vue files`);
  }
}

function isHydrated(fixtureDir: string): boolean {
  return fs.existsSync(fixtureDir) && fs.readdirSync(fixtureDir).length > 0;
}

function writeEvidence(
  selection: ReturnType<typeof selectProjects>,
  projects: ProjectEvidence[],
  grammars: TextMateGrammarEvidence[],
): void {
  const outputDir = process.env.FIXTURE_REPORT_DIR;
  if (outputDir == null || outputDir.length === 0) return;
  const resolvedOutputDir = path.resolve(root, outputDir);
  fs.mkdirSync(resolvedOutputDir, { recursive: true });
  const artifact = {
    schema: "vize.fixtureSyntaxHighlighterReport",
    version: 1,
    generatedAt: new Date().toISOString(),
    commitSha: resolveCommitSha(),
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
    path.join(resolvedOutputDir, "syntax-highlighter-summary.json"),
    `${JSON.stringify(artifact, null, 2)}\n`,
  );
}

function resolveCommitSha(): string {
  const environmentSha = process.env.GITHUB_SHA;
  if (environmentSha != null) return requireCommitSha(environmentSha, "GITHUB_SHA");
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
