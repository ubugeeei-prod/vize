// Corpus sweep + reduction (Davinci P0-5, TS-11). `corpus-baseline` and
// `corpus-diff` both run the existing real-project harness
// (`tools/support/compat/fixtures/tool-matrix-report.mjs`) across all shards through
// `runMatrix`, then reduce every per-project per-surface payload to a
// `{surface, project, file_count, content_hash}` row through `reduceShards`.
//
// The hash contract itself lives in corpus-baseline-contract.ts.
//
// Node builtins only. Reduction output is deterministic: stable sorts, no
// timestamps, no machine identity, no absolute paths.

import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import path from "node:path";

import { asArray, asNumber, asRecord, at, isRecord } from "../../../json-value.ts";
import type { JsonRecord } from "../../../json-value.ts";
import type { CorpusRow } from "./corpus-baseline-artifact.ts";
import { HASHED_FIELDS } from "./corpus-baseline-contract.ts";
import { assertHydratedGitlinkFixtures } from "./corpus-hydration.ts";
import { byKey } from "./ordering.ts";
import { repoRoot } from "./paths.ts";

export const MATRIX_SCRIPT = path.join(
  repoRoot,
  "tools",
  "support",
  "compat",
  "fixtures",
  "tool-matrix-report.mjs",
);

const PAYLOAD_FAILURE_FIELDS = ["spawnError", "parseError", "validationError"];

/**
 * Run the matrix harness as `shards` parallel processes (shards are serial
 * internally) and return the shard output directories. Throws unless every
 * shard exits 0 with a fully successful summary.
 */
export async function runMatrix({
  shards,
  vizeBin,
  tools,
  scratchDir,
  timeoutMs,
  log,
}: {
  shards: number;
  vizeBin: string;
  tools: readonly string[];
  scratchDir: string;
  timeoutMs?: number | null;
  log: (line: string) => void;
}): Promise<string[]> {
  mkdirSync(scratchDir, { recursive: true });
  assertHydratedGitlinkFixtures(listMatrixFixturePaths(shards));
  const runs: { index: number; outputDir: string; promise: Promise<{ code: number | null }> }[] =
    [];
  for (let index = 0; index < shards; index += 1) {
    const outputDir = path.join(scratchDir, `shard-${index}`);
    const args = [
      MATRIX_SCRIPT,
      "--shard-index",
      String(index),
      "--shard-count",
      String(shards),
      "--vize-bin",
      vizeBin,
      "--output-dir",
      outputDir,
    ];
    if (timeoutMs != null) args.push("--timeout-ms", String(timeoutMs));
    for (const tool of tools) args.push("--tool", tool);
    runs.push({ index, outputDir, promise: spawnShard(args, index, log) });
  }
  const failures: string[] = [];
  for (const run of runs) {
    const result = await run.promise;
    if (result.code !== 0) {
      failures.push(
        `shard ${run.index} exited ${result.code}${describeShardFailure(run.outputDir)}`,
      );
    }
  }
  if (failures.length > 0) {
    throw new Error(`matrix run failed:\n  ${failures.join("\n  ")}`);
  }
  return runs.map((run) => run.outputDir);
}

function listMatrixFixturePaths(shards: number): string[] {
  const fixturePaths: string[] = [];
  for (let index = 0; index < shards; index += 1) {
    const result = spawnSync(
      process.execPath,
      [
        MATRIX_SCRIPT,
        "--list-fixture-paths",
        "--shard-index",
        String(index),
        "--shard-count",
        String(shards),
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    if (result.status !== 0) {
      const detail = result.stderr.trim() || result.stdout.trim() || `exit ${result.status}`;
      throw new Error(`fixture path selection failed for shard ${index}: ${detail}`);
    }
    fixturePaths.push(...result.stdout.split("\n").filter(Boolean));
  }
  return fixturePaths;
}

function spawnShard(
  args: readonly string[],
  index: number,
  log: (line: string) => void,
): Promise<{ code: number | null }> {
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(process.execPath, args, {
      cwd: repoRoot,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stderrTail = "";
    child.stdout.on("data", (chunk: Buffer) => log(`[shard ${index}] ${String(chunk).trimEnd()}`));
    child.stderr.on("data", (chunk: Buffer) => {
      stderrTail = `${stderrTail}${chunk.toString()}`.slice(-4000);
    });
    child.on("error", rejectPromise);
    child.on("close", (code) => {
      if (code !== 0 && stderrTail.trim().length > 0) {
        log(`[shard ${index}] stderr: ${stderrTail.trim()}`);
      }
      resolvePromise({ code });
    });
  });
}

function describeShardFailure(outputDir: string): string {
  const summaryPath = path.join(outputDir, "summary.json");
  if (!existsSync(summaryPath)) return " (no summary.json written)";
  const summary = asRecord(JSON.parse(readFileSync(summaryPath, "utf8")), summaryPath);
  const failed: string[] = [];
  for (const projectValue of asArray(summary.projects, "projects")) {
    const project = asRecord(projectValue, "project");
    for (const runValue of asArray(project.runs, "runs")) {
      const run = asRecord(runValue, "run");
      if (run.status === "failed" || run.status === "missing-fixture") {
        failed.push(`${String(project.id)}/${String(run.tool)}: ${run.status}`);
      }
    }
  }
  return failed.length === 0 ? "" : ` (${failed.join(", ")})`;
}

/**
 * Reduce every shard's per-run payload files to sorted
 * `{surface, project, file_count, content_hash}` rows.
 */
export function reduceShards(shardDirs: readonly string[], tools: readonly string[]): CorpusRow[] {
  const rows: CorpusRow[] = [];
  const seenProjects = new Set<string>();
  for (const shardDir of shardDirs) {
    const summaryPath = path.join(shardDir, "summary.json");
    if (!existsSync(summaryPath)) {
      throw new Error(`shard output has no summary.json: ${shardDir}`);
    }
    const summary = asRecord(JSON.parse(readFileSync(summaryPath, "utf8")), summaryPath);
    assertCleanSummary(summary, shardDir);
    for (const projectValue of asArray(summary.projects, "projects")) {
      const project = asRecord(projectValue, "project");
      const id = String(project.id);
      if (seenProjects.has(id)) {
        throw new Error(`project ${id} appears in more than one shard`);
      }
      seenProjects.add(id);
      const runs = asArray(project.runs, "runs").map((run) => asRecord(run, "run"));
      const runTools = runs.map((run) => String(run.tool)).sort(byKey);
      const expectedTools = [...tools].sort(byKey);
      if (JSON.stringify(runTools) !== JSON.stringify(expectedTools)) {
        throw new Error(
          `project ${id} ran surfaces [${runTools.join(", ")}], expected [${expectedTools.join(", ")}]`,
        );
      }
      for (const run of runs) {
        rows.push(reduceRun(shardDir, id, run));
      }
    }
  }
  rows.sort(
    (left, right) => byKey(left.surface, right.surface) || byKey(left.project, right.project),
  );
  return rows;
}

function assertCleanSummary(summary: JsonRecord, shardDir: string): void {
  const counts = asRecord(summary.summary, "summary");
  const { okRuns, findingsRuns } = counts;
  const clean =
    counts.failedRuns === 0 &&
    counts.missingFixtureRuns === 0 &&
    counts.plannedRuns === 0 &&
    typeof okRuns === "number" &&
    typeof findingsRuns === "number" &&
    okRuns + findingsRuns === counts.runCount;
  if (!clean) {
    throw new Error(`shard summary is not clean: ${shardDir} (${JSON.stringify(counts)})`);
  }
}

function reduceRun(shardDir: string, projectId: string, run: JsonRecord): CorpusRow {
  const tool = String(run.tool);
  const payloadPath = path.join(shardDir, `${projectId}-${tool}.json`);
  if (!existsSync(payloadPath)) {
    throw new Error(`run payload is missing: ${payloadPath}`);
  }
  const payload = asRecord(JSON.parse(readFileSync(payloadPath, "utf8")), payloadPath);
  for (const field of PAYLOAD_FAILURE_FIELDS) {
    const failure = payload[field];
    if (failure != null) {
      const detail = typeof failure === "string" ? failure : JSON.stringify(failure);
      throw new Error(`${projectId}/${tool} payload carries ${field}: ${detail}`);
    }
  }
  const hashedFields = Object.hasOwn(HASHED_FIELDS, tool) ? HASHED_FIELDS[tool] : undefined;
  if (hashedFields == null) throw new Error(`unsupported surface: ${tool}`);
  const content: JsonRecord = {};
  for (const field of hashedFields) {
    if (!(field in payload)) {
      throw new Error(`${projectId}/${tool} payload has no ${field} field`);
    }
    content[field] = payload[field];
  }
  const fileCount = payloadFileCount(tool, payload);
  if (run.fileCount !== fileCount) {
    throw new Error(
      `${projectId}/${tool} summary fileCount ${String(run.fileCount)} != payload-derived ${fileCount}`,
    );
  }
  return {
    surface: tool,
    project: projectId,
    file_count: fileCount,
    content_hash: sha256(canonicalJson(content)),
  };
}

/** Mirrors tools/fixtures/tool-matrix-metrics.ts, from the payload side. */
function payloadFileCount(tool: string, payload: JsonRecord): number {
  const count = (...keys: string[]): number => asNumber(at(payload, ...keys), `${tool} file count`);
  if (tool === "compiler") return count("compilerArtifacts", "inputFileCount");
  if (tool === "typechecker") return count("parsed", "fileCount");
  if (tool === "linter") return asArray(payload.parsed, "linter parsed").length;
  if (tool === "formatter") return count("formatterCheck", "checkedFileCount");
  throw new Error(`unsupported surface: ${tool}`);
}

export function sha256(text: string): string {
  return createHash("sha256").update(text).digest("hex");
}

/** JSON with every object's keys sorted, so hashes ignore insertion order. */
export function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (isRecord(value)) {
    const keys = Object.keys(value).sort(byKey);
    return `{${keys.map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

export function resolveVizeBin(vizeBin: string | null | undefined): string {
  const resolved = path.resolve(repoRoot, vizeBin ?? path.join("target", "release", "vize"));
  if (!existsSync(resolved)) {
    throw new Error(
      `vize binary not found: ${resolved} (build with: cargo build --release -p vize)`,
    );
  }
  return resolved;
}

export function scratchRoot(label: string): string {
  return path.join(repoRoot, ".vize", "davinci-corpus", label);
}

export function cleanupScratch(dir: string): void {
  rmSync(dir, { recursive: true, force: true });
}
