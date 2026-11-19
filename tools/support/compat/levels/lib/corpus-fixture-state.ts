import { spawnSync } from "node:child_process";
import { readdirSync, rmSync } from "node:fs";
import path from "node:path";

import { repoRoot } from "./paths.ts";

const FIXTURE_ROOT = "tests/_fixtures/_git";
const MAX_FIXTURE_DEPTH = 3;

interface DriftedSubmodule {
  path: string;
  reason: string;
}

export function cleanFixtures(root: string = repoRoot): number {
  const targets = materializedNodeModules(root);
  for (const relativePath of targets) {
    rmSync(path.join(root, relativePath), { recursive: true, force: true });
  }
  return targets.length;
}

export function assertFixturesPristine(allowMaterialized: boolean, root: string = repoRoot): void {
  const drifted = driftedSubmodules(root);
  const materialized = allowMaterialized ? [] : materializedNodeModules(root);
  if (drifted.length === 0 && materialized.length === 0) return;

  const lines = ["corpus fixtures are not at their pinned state:"];
  if (drifted.length > 0) {
    lines.push(`  ${drifted.length} submodule(s) drifted from the recorded sha:`);
    lines.push(...drifted.slice(0, 5).map((entry) => `    ${entry.path} - ${entry.reason}`));
    if (drifted.length > 5) lines.push(`    ... and ${drifted.length - 5} more`);
    lines.push(
      "  hydrate them with:",
      `  git submodule update --init --checkout --force -- ${FIXTURE_ROOT}`,
    );
  }
  if (materialized.length > 0) {
    lines.push(
      `  ${materialized.length} materialized node_modules directory(ies) left by a previous run:`,
      ...materialized.slice(0, 5).map((relativePath) => `    ${relativePath}`),
    );
    if (materialized.length > 5) lines.push(`    ... and ${materialized.length - 5} more`);
    lines.push(
      "  clean them with `--clean-fixtures`, or by hand:",
      `  find ${FIXTURE_ROOT} -type d -name node_modules -prune -exec rm -rf {} +`,
    );
  }
  lines.push(
    "  a sweep over contaminated fixtures measures a different tree than the",
    "  baseline did - see docs/davinci/plan/corpus-baseline-notes.md, Re-record 2",
    "  (pass --allow-dirty-fixtures to sweep anyway; the hashes are then not",
    "  comparable to the committed baseline)",
  );
  throw new Error(lines.join("\n"));
}

export function materializedNodeModules(root: string = repoRoot): string[] {
  const fixtureRoot = path.join(root, FIXTURE_ROOT);
  const found: string[] = [];
  walkNodeModules(root, fixtureRoot, 0, found);
  return found.sort();
}

function driftedSubmodules(root: string): DriftedSubmodule[] {
  const result = spawnSync("git", ["submodule", "status", "--", FIXTURE_ROOT], {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.status !== 0) return [];

  const drifted: DriftedSubmodule[] = [];
  for (const line of result.stdout.split("\n")) {
    const reason = submoduleDriftReason(line[0]);
    if (reason === undefined) continue;
    const fields = line.slice(1).trim().split(/\s+/u);
    if (fields.length >= 2) drifted.push({ path: fields[1], reason });
  }
  return drifted.sort((left, right) => left.path.localeCompare(right.path));
}

function submoduleDriftReason(marker: string | undefined): string | undefined {
  if (marker === "-") return "not initialized";
  if (marker === "+") return "checked out at a different sha";
  if (marker === "U") return "has merge conflicts";
  return undefined;
}

function walkNodeModules(root: string, directory: string, depth: number, found: string[]): void {
  let entries;
  try {
    entries = readdirSync(directory, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    if (!entry.isDirectory() && !entry.isSymbolicLink()) continue;
    const entryPath = path.join(directory, entry.name);
    if (entry.name === "node_modules") {
      found.push(path.relative(root, entryPath));
      continue;
    }
    if (entry.name === ".git" || depth >= MAX_FIXTURE_DEPTH) continue;
    walkNodeModules(root, entryPath, depth + 1, found);
  }
}
