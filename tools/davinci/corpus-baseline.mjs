#!/usr/bin/env node
// Corpus baseline snapshot (Davinci P0-5).
//
// Runs `tools/fixtures/tool-matrix-report.mjs` across all shards (parallel
// shard processes; the harness is serial inside a shard), reduces every
// project x surface payload to a `{surface, project, file_count,
// content_hash}` row, and writes the committed fingerprint artifact
// `tests/_fixtures/davinci-baseline.json` together with its scope proof
// (manifest project count, projects actually run, surfaces per project).
//
// The artifact is the anchor every later Davinci phase diffs against via
// `tools/davinci/corpus-diff.mjs` (TS-11). Hash contract and the filed
// nondeterminism notes live in davinci-road/plan/corpus-baseline-notes.md.
//
// Usage:
//   node tools/davinci/corpus-baseline.mjs [options]
//     --shards <n>       parallel shard processes (default 4)
//     --vize-bin <path>  vize executable (default target/release/vize)
//     --out <path>       artifact destination (default the committed path)
//     --timeout-ms <n>   per-run timeout forwarded to the harness
//     --keep-raw         keep raw shard reports under .vize/davinci-corpus/
//
// Node builtins only. Output is deterministic: stable sort, no timestamps,
// no machine identity, no absolute paths.

import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import { buildArtifact, renderArtifact, verifyScope } from "./lib/corpus-baseline-artifact.mjs";
import {
  BASELINE_PATH,
  BASELINE_REL,
  SURFACES,
  loadManifest,
} from "./lib/corpus-baseline-contract.mjs";
import {
  cleanupScratch,
  reduceShards,
  resolveVizeBin,
  runMatrix,
  scratchRoot,
} from "./lib/corpus-baseline-run.mjs";
import { assertFixturesPristine, cleanFixtures } from "./lib/corpus-fixture-hygiene.mjs";
import { repoRoot } from "./lib/paths.mjs";

function parseArgs(argv) {
  const args = {
    shards: 4,
    vizeBin: null,
    out: null,
    timeoutMs: null,
    keepRaw: false,
    cleanFixtures: false,
    allowDirtyFixtures: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${arg} requires a value`);
      return argv[++index];
    };
    if (arg === "--shards") args.shards = positiveInteger(value(), arg);
    else if (arg === "--vize-bin") args.vizeBin = value();
    else if (arg === "--out") args.out = value();
    else if (arg === "--timeout-ms") args.timeoutMs = positiveInteger(value(), arg);
    else if (arg === "--keep-raw") args.keepRaw = true;
    else if (arg === "--clean-fixtures") args.cleanFixtures = true;
    else if (arg === "--allow-dirty-fixtures") args.allowDirtyFixtures = true;
    else if (arg === "--help" || arg === "-h") {
      process.stdout.write(
        "usage: node tools/davinci/corpus-baseline.mjs [--shards n] [--vize-bin path] [--out path] [--timeout-ms n] [--keep-raw] [--clean-fixtures] [--allow-dirty-fixtures]\n",
      );
      process.exit(0);
    } else throw new Error(`unknown argument: ${arg}`);
  }
  return args;
}

function positiveInteger(raw, name) {
  const parsed = Number(raw);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const manifest = loadManifest();
  const vizeBin = resolveVizeBin(args.vizeBin);
  const outPath = args.out == null ? BASELINE_PATH : path.resolve(args.out);
  const scratchDir = scratchRoot(`baseline-${process.pid}`);
  const log = (line) => process.stdout.write(`${line}\n`);

  if (args.cleanFixtures) {
    const removed = cleanFixtures();
    log(`corpus-baseline: cleaned ${removed} materialized node_modules from the fixtures`);
  }
  assertFixturesPristine(
    (lines) => {
      throw new Error(lines.join("\n"));
    },
    { allowMaterialized: args.allowDirtyFixtures },
  );

  log(
    `corpus-baseline: ${manifest.projects.length} projects x ${SURFACES.length} surfaces, ${args.shards} parallel shards`,
  );
  const startedAt = Date.now();
  let shardDirs;
  try {
    shardDirs = await runMatrix({
      shards: args.shards,
      vizeBin,
      tools: SURFACES,
      scratchDir,
      timeoutMs: args.timeoutMs,
      log,
    });
  } catch (error) {
    log(`raw shard reports kept for debugging: ${path.relative(repoRoot, scratchDir)}`);
    throw error;
  }
  const rows = reduceShards(shardDirs, SURFACES);
  const artifact = buildArtifact(rows, manifest);
  const scopeFailures = verifyScope(artifact, manifest, SURFACES, "generated artifact");
  if (scopeFailures.length > 0) {
    log(`raw shard reports kept for debugging: ${path.relative(repoRoot, scratchDir)}`);
    throw new Error(`scope proof failed:\n  ${scopeFailures.join("\n  ")}`);
  }
  mkdirSync(path.dirname(outPath), { recursive: true });
  writeFileSync(outPath, renderArtifact(artifact));
  if (!args.keepRaw) cleanupScratch(scratchDir);
  else log(`raw shard reports kept: ${path.relative(repoRoot, scratchDir)}`);

  const elapsedSeconds = Math.round((Date.now() - startedAt) / 1000);
  const outLabel = outPath === BASELINE_PATH ? BASELINE_REL : path.relative(repoRoot, outPath);
  log(
    `wrote ${outLabel}: ${artifact.scope.row_count} rows (${artifact.scope.projects_run} projects x ${artifact.scope.surfaces_per_project} surfaces, ${artifact.scope.total_file_count} files) in ${elapsedSeconds}s`,
  );
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
