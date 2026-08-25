#!/usr/bin/env node
// Consumer migration surface inventory for the Davinci rollout plan.
//
// The scan is intentionally observational: it records where the compiler,
// linter, typechecker, typechecker content-mapper, formatter, and LSP still
// name Davinci/S0/S1/S2, legacy AST/parser/Croquis crates, or raw OXC crates.
// It does not change runtime wiring and is safe to merge before any rollout.
//
// Usage:
//   node tools/davinci/consumer-migration-surfaces.mjs --write
//   node tools/davinci/consumer-migration-surfaces.mjs --check

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { repoRoot } from "./lib/paths.mjs";
import {
  renderConsumerMigrationSurfaceRows,
  renderConsumerMigrationSurfaces,
} from "./lib/consumer-migration-render.mjs";
import { scanConsumerMigrationSurfaces } from "./lib/consumer-migration-scan.mjs";

const ARTIFACT_REL = "davinci-road/plan/consumer-migration-surfaces.md";
const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
const ROWS_REL = "davinci-road/plan/consumer-migration-surfaces.tsv";
const ROWS = path.join(repoRoot, ROWS_REL);
const REGEN_COMMAND = "node tools/davinci/consumer-migration-surfaces.mjs --write";

function generate() {
  const scan = scanConsumerMigrationSurfaces();
  return {
    markdown: renderConsumerMigrationSurfaces(scan, {
      artifactRel: ARTIFACT_REL,
      rowsRel: ROWS_REL,
      regenCommand: REGEN_COMMAND,
    }),
    rows: renderConsumerMigrationSurfaceRows(scan),
  };
}

function artifactSet(generated) {
  return [
    { relPath: ARTIFACT_REL, path: ARTIFACT, generated: generated.markdown },
    { relPath: ROWS_REL, path: ROWS, generated: generated.rows },
  ];
}

function reportStale(artifact, committed) {
  const { relPath, generated } = artifact;
  const committedLines = committed.split("\n");
  const generatedLines = generated.split("\n");
  let firstDiff = -1;
  const max = Math.max(committedLines.length, generatedLines.length);
  for (let i = 0; i < max; i++) {
    if (committedLines[i] !== generatedLines[i]) {
      firstDiff = i;
      break;
    }
  }
  const committedSet = new Set(committedLines);
  const generatedSet = new Set(generatedLines);
  const removed = committedLines.filter((line) => !generatedSet.has(line)).length;
  const added = generatedLines.filter((line) => !committedSet.has(line)).length;
  console.error(`stale: ${relPath} drifted from the current sources.`);
  console.error(
    `  first differing line: ${firstDiff + 1} (committed ${committedLines.length} lines, regenerated ${generatedLines.length})`,
  );
  if (firstDiff >= 0) {
    console.error(`  - ${(committedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
    console.error(`  + ${(generatedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
  }
  console.error(`  lines only in committed: ${removed}, only in regenerated: ${added}`);
  console.error(`  Regenerate with: ${REGEN_COMMAND}`);
}

function writeArtifacts(generated) {
  for (const artifact of artifactSet(generated)) {
    writeFileSync(artifact.path, artifact.generated);
    console.log(`wrote ${artifact.relPath}`);
  }
}

function checkArtifacts(generated) {
  let stale = false;
  for (const artifact of artifactSet(generated)) {
    if (!existsSync(artifact.path)) {
      console.error(`stale: ${artifact.relPath} does not exist. Regenerate with: ${REGEN_COMMAND}`);
      stale = true;
      continue;
    }
    const committed = readFileSync(artifact.path, "utf8");
    if (committed !== artifact.generated) {
      reportStale(artifact, committed);
      stale = true;
    }
  }
  if (stale) process.exit(1);
  console.log(`${ARTIFACT_REL} and ${ROWS_REL} are up to date`);
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: node tools/davinci/consumer-migration-surfaces.mjs --write | --check");
    process.exit(2);
  }

  const generated = generate();
  if (mode === "--write") {
    writeArtifacts(generated);
    return;
  }

  checkArtifacts(generated);
}

main();
