// Byte-exact `--write` / `--check` for a generated artifact *set*: a list of
// files plus the shard directories the generator owns outright.
//
// Davinci plan matrices are sharded (one file per crate / consumer) so two PRs
// touching different crates never edit the same committed file. The staleness
// gate stays exactly as strict as the single-file one: every generated file
// must exist and match byte for byte, and an owned shard directory must hold
// exactly the generated shards — a leftover shard (a crate that stopped
// consuming) is stale too, and `--write` deletes it.

import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

import { byKey } from "./ordering.mjs";
import { repoRoot } from "./paths.mjs";

function absolute(relPath) {
  return path.join(repoRoot, ...relPath.split("/"));
}

function listOwnedFiles(dirRel) {
  const files = [];
  const visit = (rel) => {
    const abs = absolute(rel);
    if (!existsSync(abs)) return;
    for (const entry of readdirSync(abs, { withFileTypes: true })) {
      const child = `${rel}/${entry.name}`;
      if (entry.isDirectory()) visit(child);
      else files.push(child);
    }
  };
  visit(dirRel);
  return files.sort(byKey);
}

function removeEmptyDirs(dirRel) {
  const abs = absolute(dirRel);
  if (!existsSync(abs)) return;
  for (const entry of readdirSync(abs, { withFileTypes: true })) {
    if (entry.isDirectory()) removeEmptyDirs(`${dirRel}/${entry.name}`);
  }
  if (readdirSync(abs).length === 0) rmSync(abs, { recursive: true });
}

function fileCount(set) {
  return `${set.files.length} file${set.files.length === 1 ? "" : "s"}`;
}

function assertWellFormed(set) {
  const seen = new Set();
  for (const file of set.files) {
    if (seen.has(file.relPath)) throw new Error(`duplicate generated path ${file.relPath}`);
    seen.add(file.relPath);
  }
  return seen;
}

function strayFiles(set, wanted) {
  return (set.ownedDirs ?? []).flatMap((dir) =>
    listOwnedFiles(dir).filter((relPath) => !wanted.has(relPath)),
  );
}

function reportDrift(relPath, committed, generated, regenCommand) {
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
  console.error(`  Regenerate with: ${regenCommand}`);
}

/**
 * Write every file of the set and delete stray files in its owned directories.
 *
 * @param {{ label: string, files: { relPath: string, text: string }[], ownedDirs?: string[] }} set
 */
export function writeArtifactSet(set) {
  const wanted = assertWellFormed(set);
  for (const relPath of strayFiles(set, wanted)) {
    rmSync(absolute(relPath));
    console.log(`removed stale shard ${relPath}`);
  }
  for (const dir of set.ownedDirs ?? []) removeEmptyDirs(dir);
  for (const file of set.files) {
    const target = absolute(file.relPath);
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, file.text);
  }
  console.log(`wrote ${set.label} (${fileCount(set)})`);
}

/**
 * Byte-compare every file of the set against the committed tree; exits 1 on
 * any drift, missing file, or stray file in an owned directory.
 *
 * @param {{ label: string, files: { relPath: string, text: string }[], ownedDirs?: string[], regenCommand: string }} set
 */
export function checkArtifactSet(set) {
  const wanted = assertWellFormed(set);
  let stale = 0;
  for (const file of set.files) {
    const target = absolute(file.relPath);
    if (!existsSync(target)) {
      console.error(`stale: ${file.relPath} does not exist. Regenerate with: ${set.regenCommand}`);
      stale++;
      continue;
    }
    const committed = readFileSync(target, "utf8");
    if (committed !== file.text) {
      reportDrift(file.relPath, committed, file.text, set.regenCommand);
      stale++;
    }
  }
  for (const relPath of strayFiles(set, wanted)) {
    console.error(
      `stale: ${relPath} is not produced by the generator (a leftover shard). Regenerate with: ${set.regenCommand}`,
    );
    stale++;
  }
  if (stale > 0) process.exit(1);
  console.log(`${set.label} is up to date (${fileCount(set)})`);
}
