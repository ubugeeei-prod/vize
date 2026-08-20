#!/usr/bin/env node
// Corpus baseline diff gate (Davinci P0-5, suite TS-11).
//
// Runs a fresh whole-corpus sweep through the same reduction as
// `tools/davinci/corpus-baseline.mjs` and compares it against the committed
// baseline `tests/_fixtures/davinci-baseline.json`, reporting per-surface
// per-project drift.
//
// Exit 0 requires BOTH TS-11 conditions:
//   (a) zero drift between the fresh run and the committed baseline, and
//   (b) the scope proof holds — the baseline artifact exists and both the
//       baseline and the fresh run cover every project in
//       tests/_fixtures/vue-ecosystem-fixtures.json on every gated surface
//       with a nonzero total file count (a zero-file run fails).
// A missing baseline, a scope shortfall, or any drift exits 1 with the
// exact reasons.
//
// Usage:
//   node tools/davinci/corpus-diff.mjs [options]
//     --surface <s[,s]>  gate only these surfaces (compiler, formatter,
//                        linter, typechecker); the committed baseline is
//                        still validated against the full manifest scope
//     --shards <n>       parallel shard processes (default 4)
//     --vize-bin <path>  vize executable (default target/release/vize)
//     --baseline <path>  baseline artifact (default the committed path)
//     --write-fresh <p>  also write the fresh run's artifact to this path
//     --timeout-ms <n>   per-run timeout forwarded to the harness
//     --keep-raw         keep raw shard reports under .vize/davinci-corpus/
//
// Node builtins only.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  buildArtifact,
  diffRows,
  renderArtifact,
  verifyScope,
} from "./lib/corpus-baseline-artifact.mjs";
import {
  BASELINE_PATH,
  SURFACES,
  UNSTABLE_REL,
  loadManifest,
  loadUnstableRows,
  parseSurfaceFilter,
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
    baseline: null,
    writeFresh: null,
    timeoutMs: null,
    keepRaw: false,
    surfaces: [],
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
    else if (arg === "--baseline") args.baseline = value();
    else if (arg === "--write-fresh") args.writeFresh = value();
    else if (arg === "--timeout-ms") args.timeoutMs = positiveInteger(value(), arg);
    else if (arg === "--keep-raw") args.keepRaw = true;
    else if (arg === "--clean-fixtures") args.cleanFixtures = true;
    else if (arg === "--allow-dirty-fixtures") args.allowDirtyFixtures = true;
    else if (arg === "--surface") {
      args.surfaces.push(
        ...value()
          .split(",")
          .map((part) => part.trim())
          .filter(Boolean),
      );
    } else if (arg === "--help" || arg === "-h") {
      process.stdout.write(
        "usage: node tools/davinci/corpus-diff.mjs [--surface s[,s]] [--shards n] [--vize-bin path] [--baseline path] [--write-fresh path] [--timeout-ms n] [--keep-raw] [--clean-fixtures] [--allow-dirty-fixtures]\n",
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
  const surfaces = parseSurfaceFilter(args.surfaces);
  const manifest = loadManifest();
  const baselinePath = args.baseline == null ? BASELINE_PATH : path.resolve(args.baseline);
  const baselineLabel = path.relative(repoRoot, baselinePath);
  const log = (line) => process.stdout.write(`${line}\n`);

  if (args.cleanFixtures) {
    const removed = cleanFixtures();
    log(`corpus-diff: cleaned ${removed} materialized node_modules from the fixtures`);
  }
  assertFixturesPristine(fail, { allowMaterialized: args.allowDirtyFixtures });

  if (!existsSync(baselinePath)) {
    fail([
      `baseline artifact is missing: ${baselineLabel}`,
      "generate it with: node tools/davinci/corpus-baseline.mjs",
    ]);
  }
  let baseline;
  try {
    baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
  } catch (error) {
    fail([`baseline artifact is not valid JSON: ${baselineLabel} (${error.message})`]);
  }
  const baselineScopeFailures = verifyScope(baseline, manifest, SURFACES, "committed baseline");
  if (baselineScopeFailures.length > 0) {
    fail(["scope proof failed for the committed baseline:", ...baselineScopeFailures]);
  }
  const vizeBin = resolveVizeBin(args.vizeBin);

  log(
    `corpus-diff: ${manifest.projects.length} projects, surfaces [${surfaces.join(", ")}], ${args.shards} parallel shards`,
  );
  const startedAt = Date.now();
  const scratchDir = scratchRoot(`diff-${process.pid}`);
  let shardDirs;
  try {
    shardDirs = await runMatrix({
      shards: args.shards,
      vizeBin,
      tools: surfaces,
      scratchDir,
      timeoutMs: args.timeoutMs,
      log,
    });
  } catch (error) {
    log(`raw shard reports kept for debugging: ${path.relative(repoRoot, scratchDir)}`);
    throw error;
  }
  const freshRows = reduceShards(shardDirs, surfaces);
  const fresh = buildArtifact(freshRows, manifest);
  if (args.writeFresh != null) {
    const freshPath = path.resolve(args.writeFresh);
    mkdirSync(path.dirname(freshPath), { recursive: true });
    writeFileSync(freshPath, renderArtifact(fresh));
    log(`wrote fresh artifact: ${path.relative(repoRoot, freshPath)}`);
  }
  if (!args.keepRaw) cleanupScratch(scratchDir);
  else log(`raw shard reports kept: ${path.relative(repoRoot, scratchDir)}`);

  const freshScopeFailures = verifyScope(fresh, manifest, surfaces, "fresh run");
  const baselineRows = baseline.rows.filter((row) => surfaces.includes(row.surface));
  const allDrift = diffRows(baselineRows, freshRows);
  const unstableKeys = new Set(
    loadUnstableRows(manifest).map((row) => `${row.surface} ${row.project}`),
  );
  const drift = allDrift.filter(
    (record) =>
      record.kind !== "changed" || !unstableKeys.has(`${record.surface} ${record.project}`),
  );
  const unstableDrift = allDrift.filter((record) => !drift.includes(record));
  const elapsedSeconds = Math.round((Date.now() - startedAt) / 1000);

  for (const record of unstableDrift) {
    log(
      `unstable (filed in ${UNSTABLE_REL}, not gating): ${record.surface}/${record.project} ${record.baseline_hash.slice(0, 12)} -> ${record.fresh_hash.slice(0, 12)}`,
    );
  }
  if (drift.length > 0) {
    log(`drift: ${drift.length} row(s) differ from ${baselineLabel}`);
    for (const record of drift) {
      if (record.kind === "changed") {
        const fileNote =
          record.baseline_file_count === record.fresh_file_count
            ? `${record.fresh_file_count} files`
            : `files ${record.baseline_file_count} -> ${record.fresh_file_count}`;
        log(
          `  changed  ${record.surface}/${record.project} (${fileNote}) ${record.baseline_hash.slice(0, 12)} -> ${record.fresh_hash.slice(0, 12)}`,
        );
      } else {
        log(`  ${record.kind}  ${record.surface}/${record.project}`);
      }
    }
    const bySurface = new Map();
    for (const record of drift) {
      bySurface.set(record.surface, (bySurface.get(record.surface) ?? 0) + 1);
    }
    log(
      `drift by surface: ${[...bySurface.entries()]
        .map(([surface, count]) => `${surface}=${count}`)
        .join(", ")}`,
    );
  }
  if (freshScopeFailures.length > 0) {
    log("scope proof failed for the fresh run:");
    for (const reason of freshScopeFailures) log(`  ${reason}`);
  }
  if (drift.length > 0 || freshScopeFailures.length > 0) {
    log(`corpus-diff: FAIL in ${elapsedSeconds}s`);
    process.exit(1);
  }
  const filterNote =
    surfaces.length === SURFACES.length ? "" : ` [surface filter: ${surfaces.join(", ")}]`;
  const unstableNote =
    unstableDrift.length === 0
      ? ""
      : ` (${unstableDrift.length} filed unstable row(s) drifted without gating)`;
  log(
    `corpus-diff: PASS in ${elapsedSeconds}s — zero gating drift across ${freshRows.length} rows (${fresh.scope.projects_run} projects x ${fresh.scope.surfaces_per_project} surfaces, ${fresh.scope.total_file_count} files); scope proof matches ${manifest.projects.length}-project manifest${filterNote}${unstableNote}`,
  );
}

function fail(reasons) {
  for (const reason of reasons) process.stderr.write(`${reason}\n`);
  process.exit(1);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
