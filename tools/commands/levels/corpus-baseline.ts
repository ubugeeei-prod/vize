import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  buildArtifact,
  expectedComparisonCount,
  renderArtifact,
  verifyScope,
} from "../../support/compat/davinci/lib/corpus-baseline-artifact.ts";
import {
  BASELINE_PATH,
  BASELINE_REL,
  loadManifest,
  parseSurfaceFilter,
} from "../../support/compat/davinci/lib/corpus-baseline-contract.ts";
import {
  assertFixturesPristine,
  cleanFixtures,
} from "../../support/compat/davinci/lib/corpus-fixture-state.ts";
import {
  cleanupScratch,
  reduceShards,
  resolveVizeBin,
  runMatrix,
  scratchRoot,
} from "../../support/compat/davinci/lib/corpus-baseline-run.ts";
import { repoRoot } from "../../support/compat/davinci/lib/paths.ts";

interface Arguments {
  shards: number;
  vizeBin?: string;
  out?: string;
  timeoutMs?: number;
  keepRaw: boolean;
  cleanFixtures: boolean;
  allowDirtyFixtures: boolean;
  help: boolean;
}

const USAGE =
  "usage: node tools/commands/davinci/corpus-baseline.ts [--shards n] [--vize-bin path] [--out path] [--timeout-ms n] [--keep-raw] [--clean-fixtures] [--allow-dirty-fixtures]";

async function main(argv: readonly string[]): Promise<void> {
  const args = parseArguments(argv);
  if (args.help) {
    process.stdout.write(`${USAGE}\n`);
    return;
  }

  const manifest = loadManifest();
  const vizeBin = resolveVizeBin(args.vizeBin);
  const outPath = args.out === undefined ? BASELINE_PATH : path.resolve(repoRoot, args.out);
  const scratchDir = scratchRoot(`baseline-${process.pid}-${Date.now()}`);

  if (args.cleanFixtures) {
    const removed = cleanFixtures();
    process.stdout.write(
      `corpus-baseline: cleaned ${removed} materialized node_modules from the fixtures\n`,
    );
  }
  assertFixturesPristine(args.allowDirtyFixtures);

  const surfaces = parseSurfaceFilter([]);
  process.stdout.write(
    `corpus-baseline: ${manifest.projects.length} projects x ${surfaces.length} surfaces, ${args.shards} parallel shards\n`,
  );
  const started = performance.now();
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

  const rows = reduceShards(shardDirs, surfaces);
  const artifact = buildArtifact(rows, manifest);
  const scopeFailures = verifyScope(artifact, manifest, surfaces, "generated artifact");
  if (scopeFailures.length > 0) {
    process.stdout.write(`raw shard reports kept for debugging: ${relativePath(scratchDir)}\n`);
    throw new Error(`scope proof failed:\n  ${scopeFailures.join("\n  ")}`);
  }

  mkdirSync(path.dirname(outPath), { recursive: true });
  writeFileSync(outPath, renderArtifact(artifact));
  if (args.keepRaw) {
    process.stdout.write(`raw shard reports kept: ${relativePath(scratchDir)}\n`);
  } else {
    cleanupScratch(scratchDir);
  }

  const elapsedSeconds = Math.floor((performance.now() - started) / 1_000);
  const outLabel = outPath === BASELINE_PATH ? BASELINE_REL : relativePath(outPath);
  const expectedComparisons = expectedComparisonCount(manifest, surfaces);
  process.stdout.write(
    `wrote ${outLabel}: ${artifact.scope.row_count}/${expectedComparisons} comparisons (${artifact.scope.projects_run} projects x ${artifact.scope.surfaces_per_project} surfaces, ${artifact.scope.total_file_count} files) in ${elapsedSeconds}s\n`,
  );
}

function parseArguments(argv: readonly string[]): Arguments {
  const args: Arguments = {
    shards: 4,
    keepRaw: false,
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
    else if (argument === "--out") args.out = value();
    else if (argument === "--timeout-ms") args.timeoutMs = positiveInteger(value(), argument);
    else if (argument === "--keep-raw") args.keepRaw = true;
    else if (argument === "--clean-fixtures") args.cleanFixtures = true;
    else if (argument === "--allow-dirty-fixtures") args.allowDirtyFixtures = true;
    else if (argument === "--help" || argument === "-h") args.help = true;
    else throw new Error(`unknown argument: ${argument}`);
  }
  return args;
}

function positiveInteger(value: string, argument: string): number {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new Error(`${argument} requires a positive integer`);
  }
  return parsed;
}

function relativePath(filePath: string): string {
  return path.relative(repoRoot, filePath) || ".";
}

void main(process.argv.slice(2)).catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
