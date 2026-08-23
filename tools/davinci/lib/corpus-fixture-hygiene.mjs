// Fixture hygiene for the davinci corpus sweeps (TS-11).
//
// A corpus run only produces comparable hashes when it starts from the
// fixtures the manifest pins: every submodule at its recorded sha, and no
// stray build state inside the checkouts. Two things break that in practice:
//
//   - `vize check` materializes tsgo project state, including a `node_modules`
//     directory, *inside* the checked project. One sweep leaves ~142 fixture
//     projects dirty, and the next sweep then measures a different tree than
//     the baseline did (davinci-road/plan/corpus-baseline-notes.md,
//     "Re-record 2" — that inheritance produced drift on two surfaces and cost
//     a full investigation before it was understood).
//   - a partially-hydrated or drifted submodule silently narrows the corpus,
//     which the scope proof catches only when a project disappears entirely.
//
// The failure mode both share is silence: the sweep completes, prints hashes,
// and nothing says the inputs were not the pinned ones. This module makes the
// inputs an explicit precondition instead.
//
// Node builtins only; output is deterministic (sorted paths, no timestamps).

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { repoRoot } from "./paths.mjs";

export const FIXTURE_ROOT = path.join("tests", "_fixtures", "_git");

/** Submodule states `git submodule status` marks as not-at-the-recorded-sha. */
const DIRTY_PREFIXES = new Map([
  ["-", "not initialized"],
  ["+", "checked out at a different sha"],
  ["U", "has merge conflicts"],
]);

function git(args) {
  const result = spawnSync("git", args, { cwd: repoRoot, encoding: "utf8" });
  if (result.error != null) throw result.error;
  return result;
}

/**
 * Submodules under the fixture root that are not at their recorded sha.
 * Returns `[{ path, reason }]`, sorted by path.
 */
export function driftedSubmodules() {
  const result = git(["submodule", "status", "--", FIXTURE_ROOT]);
  if (result.status !== 0) return [];
  const drifted = [];
  for (const line of result.stdout.split("\n")) {
    if (line === "") continue;
    const reason = DIRTY_PREFIXES.get(line[0]);
    if (reason == null) continue;
    // ` <sha> <path> (<describe>)` — the path is the second field.
    const fields = line.slice(1).trim().split(/\s+/);
    if (fields.length < 2) continue;
    drifted.push({ path: fields[1], reason });
  }
  return drifted.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

/**
 * `node_modules` directories materialized inside fixture checkouts, sorted.
 * Only the shallow levels a tool run can create are scanned, so a legitimately
 * committed deep `node_modules` fixture stays out of the result.
 */
export function materializedNodeModules(maxDepth = 3) {
  const root = path.join(repoRoot, FIXTURE_ROOT);
  const found = [];
  const walk = (dir, depth) => {
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const full = path.join(dir, entry.name);
      if (entry.name === "node_modules") {
        found.push(path.relative(repoRoot, full));
        continue; // never descend into one
      }
      if (entry.name === ".git" || depth >= maxDepth) continue;
      walk(full, depth + 1);
    }
  };
  walk(root, 0);
  return found.sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
}

/** Deletes every path `materializedNodeModules` reports. Returns the count. */
export function cleanFixtures() {
  const targets = materializedNodeModules();
  for (const relative of targets) {
    fs.rmSync(path.join(repoRoot, relative), { recursive: true, force: true });
  }
  return targets.length;
}

const CLEAN_HINT =
  "clean them with `--clean-fixtures`, or by hand:\n" +
  `  find ${FIXTURE_ROOT} -type d -name node_modules -prune -exec rm -rf {} +`;

const HYDRATE_HINT = `  git submodule update --init --checkout --force -- ${FIXTURE_ROOT}`;

/**
 * Fails loudly when the fixtures are not the pinned ones.
 *
 * `fail` is the caller's reporter and takes the message as an array of lines,
 * matching both corpus tools. A contaminated tree stops the run *before* it
 * spends minutes producing hashes nobody can trust.
 */
export function assertFixturesPristine(fail, { allowMaterialized = false } = {}) {
  const drifted = driftedSubmodules();
  const materialized = allowMaterialized ? [] : materializedNodeModules();
  if (drifted.length === 0 && materialized.length === 0) return;

  const lines = ["corpus fixtures are not at their pinned state:"];
  if (drifted.length > 0) {
    lines.push(`  ${drifted.length} submodule(s) drifted from the recorded sha:`);
    for (const { path: submodule, reason } of drifted.slice(0, 5)) {
      lines.push(`    ${submodule} — ${reason}`);
    }
    if (drifted.length > 5) lines.push(`    … and ${drifted.length - 5} more`);
    lines.push("  hydrate them with:");
    lines.push(HYDRATE_HINT);
  }
  if (materialized.length > 0) {
    lines.push(
      `  ${materialized.length} materialized node_modules directory(ies) left by a previous run:`,
    );
    for (const relative of materialized.slice(0, 5)) lines.push(`    ${relative}`);
    if (materialized.length > 5) lines.push(`    … and ${materialized.length - 5} more`);
    lines.push(`  ${CLEAN_HINT}`);
  }
  lines.push(
    "  a sweep over contaminated fixtures measures a different tree than the",
    "  baseline did — see davinci-road/plan/corpus-baseline-notes.md, Re-record 2",
    "  (pass --allow-dirty-fixtures to sweep anyway; the hashes are then not",
    "  comparable to the committed baseline)",
  );
  fail(lines.flatMap((line) => line.split("\n")));
}
