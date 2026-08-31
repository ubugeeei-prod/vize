// Corpus sweep + reduction (Davinci P0-5, TS-11). `corpus-baseline.mjs` and
// `corpus-diff.mjs` both run the existing real-project harness
// (`tools/fixtures/tool-matrix-report.mjs`) across all shards through
// `runMatrix`, then reduce every per-project per-surface payload to a
// `{surface, project, file_count, content_hash}` row through `reduceShards`.
//
// The hash contract itself lives in corpus-baseline-contract.mjs.
//
// Node builtins only. Reduction output is deterministic: stable sorts, no
// timestamps, no machine identity, no absolute paths.

import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import path from "node:path";

import { HASHED_FIELDS } from "./corpus-baseline-contract.mjs";
import { assertHydratedGitlinkFixtures } from "./corpus-hydration.mjs";
import { byKey } from "./ordering.mjs";
import { repoRoot } from "./paths.mjs";

export const MATRIX_SCRIPT = path.join(repoRoot, "tools", "fixtures", "tool-matrix-report.mjs");

const PAYLOAD_FAILURE_FIELDS = ["spawnError", "parseError", "validationError"];

/**
 * Run the matrix harness as `shards` parallel processes (shards are serial
 * internally) and return the shard output directories. Throws unless every
 * shard exits 0 with a fully successful summary.
 */
export async function runMatrix({ shards, vizeBin, tools, scratchDir, timeoutMs, log }) {
  mkdirSync(scratchDir, { recursive: true });
  assertHydratedGitlinkFixtures(listMatrixFixturePaths(shards));
  const runs = [];
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
  const failures = [];
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

function listMatrixFixturePaths(shards) {
  const fixturePaths = [];
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

function spawnShard(args, index, log) {
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(process.execPath, args, {
      cwd: repoRoot,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stderrTail = "";
    child.stdout.on("data", (chunk) => log(`[shard ${index}] ${String(chunk).trimEnd()}`));
    child.stderr.on("data", (chunk) => {
      stderrTail = `${stderrTail}${chunk}`.slice(-4000);
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

function describeShardFailure(outputDir) {
  const summaryPath = path.join(outputDir, "summary.json");
  if (!existsSync(summaryPath)) return " (no summary.json written)";
  const summary = JSON.parse(readFileSync(summaryPath, "utf8"));
  const failed = [];
  for (const project of summary.projects) {
    for (const run of project.runs) {
      if (run.status === "failed" || run.status === "missing-fixture") {
        failed.push(`${project.id}/${run.tool}: ${run.status}`);
      }
    }
  }
  return failed.length === 0 ? "" : ` (${failed.join(", ")})`;
}

/**
 * Reduce every shard's per-run payload files to sorted
 * `{surface, project, file_count, content_hash}` rows.
 */
export function reduceShards(shardDirs, tools) {
  const rows = [];
  const seenProjects = new Set();
  for (const shardDir of shardDirs) {
    const summaryPath = path.join(shardDir, "summary.json");
    if (!existsSync(summaryPath)) {
      throw new Error(`shard output has no summary.json: ${shardDir}`);
    }
    const summary = JSON.parse(readFileSync(summaryPath, "utf8"));
    assertCleanSummary(summary, shardDir);
    for (const project of summary.projects) {
      if (seenProjects.has(project.id)) {
        throw new Error(`project ${project.id} appears in more than one shard`);
      }
      seenProjects.add(project.id);
      const runTools = project.runs.map((run) => run.tool).sort(byKey);
      const expectedTools = [...tools].sort(byKey);
      if (JSON.stringify(runTools) !== JSON.stringify(expectedTools)) {
        throw new Error(
          `project ${project.id} ran surfaces [${runTools.join(", ")}], expected [${expectedTools.join(", ")}]`,
        );
      }
      for (const run of project.runs) {
        rows.push(reduceRun(shardDir, project.id, run));
      }
    }
  }
  rows.sort(
    (left, right) => byKey(left.surface, right.surface) || byKey(left.project, right.project),
  );
  return rows;
}

function assertCleanSummary(summary, shardDir) {
  const counts = summary.summary;
  const clean =
    counts.failedRuns === 0 &&
    counts.missingFixtureRuns === 0 &&
    counts.plannedRuns === 0 &&
    counts.okRuns + counts.findingsRuns === counts.runCount;
  if (!clean) {
    throw new Error(`shard summary is not clean: ${shardDir} (${JSON.stringify(counts)})`);
  }
}

function reduceRun(shardDir, projectId, run) {
  const payloadPath = path.join(shardDir, `${projectId}-${run.tool}.json`);
  if (!existsSync(payloadPath)) {
    throw new Error(`run payload is missing: ${payloadPath}`);
  }
  const payload = JSON.parse(readFileSync(payloadPath, "utf8"));
  for (const field of PAYLOAD_FAILURE_FIELDS) {
    if (payload[field] != null) {
      throw new Error(`${projectId}/${run.tool} payload carries ${field}: ${payload[field]}`);
    }
  }
  const hashedFields = HASHED_FIELDS[run.tool];
  if (hashedFields == null) throw new Error(`unsupported surface: ${run.tool}`);
  const content = {};
  for (const field of hashedFields) {
    if (!(field in payload)) {
      throw new Error(`${projectId}/${run.tool} payload has no ${field} field`);
    }
    content[field] = payload[field];
  }
  const fileCount = payloadFileCount(run.tool, payload);
  if (run.fileCount !== fileCount) {
    throw new Error(
      `${projectId}/${run.tool} summary fileCount ${run.fileCount} != payload-derived ${fileCount}`,
    );
  }
  return {
    surface: run.tool,
    project: projectId,
    file_count: fileCount,
    content_hash: sha256(canonicalJson(content)),
  };
}

/** Mirrors tools/fixtures/tool-matrix-metrics.mjs, from the payload side. */
function payloadFileCount(tool, payload) {
  if (tool === "compiler") return payload.compilerArtifacts.inputFileCount;
  if (tool === "typechecker") return payload.parsed.fileCount;
  if (tool === "linter") return payload.parsed.length;
  if (tool === "formatter") return payload.formatterCheck.checkedFileCount;
  throw new Error(`unsupported surface: ${tool}`);
}

export function sha256(text) {
  return createHash("sha256").update(text).digest("hex");
}

/** JSON with every object's keys sorted, so hashes ignore insertion order. */
export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    const keys = Object.keys(value).sort(byKey);
    return `{${keys.map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

export function resolveVizeBin(vizeBin) {
  const resolved = path.resolve(repoRoot, vizeBin ?? path.join("target", "release", "vize"));
  if (!existsSync(resolved)) {
    throw new Error(
      `vize binary not found: ${resolved} (build with: cargo build --release -p vize)`,
    );
  }
  return resolved;
}

export function scratchRoot(label) {
  return path.join(repoRoot, ".vize", "davinci-corpus", label);
}

export function cleanupScratch(dir) {
  rmSync(dir, { recursive: true, force: true });
}
