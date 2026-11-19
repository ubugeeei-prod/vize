import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  buildArtifact,
  diffRows,
  expectedComparisonCount,
  parseCorpusArtifact,
  renderArtifact,
  verifyScope,
} from "../../support/compat/levels/lib/corpus-baseline-artifact.ts";
import type { CorpusDrift } from "../../support/compat/levels/lib/corpus-baseline-artifact.ts";
import {
  BASELINE_PATH,
  UNSTABLE_REL,
  loadManifest,
  loadUnstableRows,
  parseSurfaceFilter,
} from "../../support/compat/levels/lib/corpus-baseline-contract.ts";
import {
  assertFixturesPristine,
  cleanFixtures,
} from "../../support/compat/levels/lib/corpus-fixture-state.ts";
import {
  cleanupScratch,
  reduceShards,
  resolveVizeBin,
  runMatrix,
  scratchRoot,
} from "../../support/compat/levels/lib/corpus-baseline-run.ts";
import { repoRoot } from "../../support/compat/levels/lib/paths.ts";

interface Arguments {
  shards: number;
  vizeBin?: string;
  baseline?: string;
  writeFresh?: string;
  timeoutMs?: number;
  keepRaw: boolean;
  surfaces: string[];
  cleanFixtures: boolean;
  allowDirtyFixtures: boolean;
  help: boolean;
}

const USAGE =
  "usage: node tools/commands/levels/corpus-diff.ts [--surface s[,s]] [--shards n] [--vize-bin path] [--baseline path] [--write-fresh path] [--timeout-ms n] [--keep-raw] [--clean-fixtures] [--allow-dirty-fixtures]";

async function main(argv: readonly string[]): Promise<void> {
  const args = parseArguments(argv);
  if (args.help) {
    process.stdout.write(`${USAGE}\n`);
    return;
  }

  const surfaces = parseSurfaceFilter(args.surfaces);
  const manifest = loadManifest();
  const baselinePath = path.resolve(repoRoot, args.baseline ?? BASELINE_PATH);
  const baselineLabel = relativePath(baselinePath);

  if (args.cleanFixtures) {
    const removed = cleanFixtures();
    process.stdout.write(
      `corpus-diff: cleaned ${removed} materialized node_modules from the fixtures\n`,
    );
  }
  assertFixturesPristine(args.allowDirtyFixtures);

  if (!existsSync(baselinePath)) {
    throw new Error(
      `baseline artifact is missing: ${baselineLabel}\ngenerate it with: node tools/commands/levels/corpus-baseline.ts`,
    );
  }
  const baseline = parseCorpusArtifact(
    JSON.parse(readFileSync(baselinePath, "utf8")) as unknown,
    baselineLabel,
  );
  const allSurfaces = parseSurfaceFilter([]);
  const baselineScopeFailures = verifyScope(baseline, manifest, allSurfaces, "committed baseline");
  if (baselineScopeFailures.length > 0) {
    throw new Error(
      `scope proof failed for the committed baseline:\n${baselineScopeFailures.join("\n")}`,
    );
  }

  const vizeBin = resolveVizeBin(args.vizeBin);
  process.stdout.write(
    `corpus-diff: ${manifest.projects.length} projects, surfaces [${surfaces.join(", ")}], ${args.shards} parallel shards\n`,
  );
  const started = performance.now();
  const scratchDir = scratchRoot(`diff-${process.pid}-${Date.now()}`);
  let shardDirs: string[];
  try {
    shardDirs = await runMatrix({
      shards: args.shards,
      vizeBin,
      tools: surfaces,
      scratchDir,
      timeoutMs: args.timeoutMs,
      log: (line) => process.stdout.write(`${line}\n`),
    });
  } catch (error) {
    process.stdout.write(`raw shard reports kept for debugging: ${relativePath(scratchDir)}\n`);
    throw error;
  }

  const freshRows = reduceShards(shardDirs, surfaces);
  const fresh = buildArtifact(freshRows, manifest);
  if (args.writeFresh !== undefined) {
    const freshPath = path.resolve(repoRoot, args.writeFresh);
    mkdirSync(path.dirname(freshPath), { recursive: true });
    writeFileSync(freshPath, renderArtifact(fresh));
    process.stdout.write(`wrote fresh artifact: ${relativePath(freshPath)}\n`);
  }
  if (args.keepRaw) {
    process.stdout.write(`raw shard reports kept: ${relativePath(scratchDir)}\n`);
  } else {
    cleanupScratch(scratchDir);
  }

  const freshScopeFailures = verifyScope(fresh, manifest, surfaces, "fresh run");
  const baselineRows = (baseline.rows ?? []).filter((row) => surfaces.includes(row.surface));
  const unstableKeys = new Set(
    loadUnstableRows(manifest).map((row) => `${row.surface} ${row.project}`),
  );
  const drift: CorpusDrift[] = [];
  const unstableDrift: CorpusDrift[] = [];
  for (const record of diffRows(baselineRows, freshRows)) {
    const destination =
      record.kind === "changed" && unstableKeys.has(`${record.surface} ${record.project}`)
        ? unstableDrift
        : drift;
    destination.push(record);
  }
  const elapsedSeconds = Math.floor((performance.now() - started) / 1_000);

  for (const record of unstableDrift) {
    if (record.kind !== "changed") continue;
    process.stdout.write(
      `unstable (filed in ${UNSTABLE_REL}, not gating): ${record.surface}/${record.project} ${prefix12(record.baseline_hash)} -> ${prefix12(record.fresh_hash)}\n`,
    );
  }
  reportDrift(drift, baselineLabel);
  if (freshScopeFailures.length > 0) {
    process.stdout.write("scope proof failed for the fresh run:\n");
    for (const reason of freshScopeFailures) process.stdout.write(`  ${reason}\n`);
  }
  if (drift.length > 0 || freshScopeFailures.length > 0) {
    process.stdout.write(`corpus-diff: FAIL in ${elapsedSeconds}s\n`);
    throw new Error("corpus-diff failed");
  }

  const expectedComparisons = expectedComparisonCount(manifest, surfaces);
  const filterNote =
    surfaces.length === allSurfaces.length ? "" : ` [surface filter: ${surfaces.join(", ")}]`;
  const unstableNote =
    unstableDrift.length === 0
      ? ""
      : ` (${unstableDrift.length} filed unstable row(s) drifted without gating)`;
  process.stdout.write(
    `corpus-diff: PASS in ${elapsedSeconds}s - zero gating drift across ${freshRows.length}/${expectedComparisons} comparisons (${fresh.scope.projects_run} projects x ${fresh.scope.surfaces_per_project} surfaces, ${fresh.scope.total_file_count} files); scope proof matches ${manifest.projects.length}-project manifest${filterNote}${unstableNote}\n`,
  );
}

function reportDrift(drift: readonly CorpusDrift[], baselineLabel: string): void {
  if (drift.length === 0) return;
  process.stdout.write(`drift: ${drift.length} row(s) differ from ${baselineLabel}\n`);
  const bySurface = new Map<string, number>();
  for (const record of drift) {
    bySurface.set(record.surface, (bySurface.get(record.surface) ?? 0) + 1);
    if (record.kind !== "changed") {
      process.stdout.write(`  ${record.kind}  ${record.surface}/${record.project}\n`);
      continue;
    }
    const fileNote =
      record.baseline_file_count === record.fresh_file_count
        ? `${record.fresh_file_count} files`
        : `files ${record.baseline_file_count} -> ${record.fresh_file_count}`;
    process.stdout.write(
      `  changed  ${record.surface}/${record.project} (${fileNote}) ${prefix12(record.baseline_hash)} -> ${prefix12(record.fresh_hash)}\n`,
    );
  }
  const summary = [...bySurface.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([surface, count]) => `${surface}=${count}`)
    .join(", ");
  process.stdout.write(`drift by surface: ${summary}\n`);
}

function parseArguments(argv: readonly string[]): Arguments {
  const args: Arguments = {
    shards: 4,
    keepRaw: false,
    surfaces: [],
    cleanFixtures: false,
    allowDirtyFixtures: false,
    help: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    const value = (): string => {
      index += 1;
      const next = argv[index];
      if (next === undefined) throw new Error(`${argument} requires a value`);
      return next;
    };
    if (argument === "--shards") args.shards = positiveInteger(value(), argument);
    else if (argument === "--vize-bin") args.vizeBin = value();
    else if (argument === "--baseline") args.baseline = value();
    else if (argument === "--write-fresh") args.writeFresh = value();
    else if (argument === "--timeout-ms") args.timeoutMs = positiveInteger(value(), argument);
    else if (argument === "--keep-raw") args.keepRaw = true;
    else if (argument === "--clean-fixtures") args.cleanFixtures = true;
    else if (argument === "--allow-dirty-fixtures") args.allowDirtyFixtures = true;
    else if (argument === "--surface") args.surfaces.push(...splitCsv(value()));
    else if (argument === "--help" || argument === "-h") args.help = true;
    else throw new Error(`unknown argument: ${argument}`);
  }
  return args;
}

function splitCsv(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function positiveInteger(value: string, argument: string): number {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new Error(`${argument} requires a positive integer`);
  }
  return parsed;
}

function prefix12(value: string): string {
  return value.slice(0, 12);
}

function relativePath(filePath: string): string {
  return path.relative(repoRoot, filePath) || ".";
}

void main(process.argv.slice(2)).catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
