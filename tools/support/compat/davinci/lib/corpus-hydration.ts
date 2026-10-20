// Hydration preflight for Davinci real-project corpus runs.
//
// The corpus runners operate on pinned gitlinks under tests/_fixtures/_git.
// A checkout that has the gitlink but not the submodule working tree must fail
// before the tool matrix starts, otherwise an empty directory can look like a
// successful zero-file corpus run.

import { spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";

import { repoRoot } from "./paths.mjs";

export function assertHydratedGitlinkFixtures(fixturePaths, root = repoRoot) {
  const uniquePaths = uniqueFixturePaths(fixturePaths);
  const failures = fixtureHydrationFailures(root, uniquePaths);
  if (failures.length === 0) return;
  const recoveryPaths = gitlinkFixturePaths(root, uniquePaths);

  throw new Error(
    [
      "corpus fixture hydration preflight failed:",
      ...failures.map((failure) => `  ${failure}`),
      ...recoveryCommandLines(recoveryPaths),
    ].join("\n"),
  );
}

export function fixtureHydrationFailures(root, fixturePaths) {
  const uniquePaths = uniqueFixturePaths(fixturePaths);
  const safePaths = uniquePaths.filter(isRelativeFixturePath);
  const gitlinks = readGitlinks(root, safePaths);
  const failures = [];
  for (const relPath of uniquePaths) {
    if (!isRelativeFixturePath(relPath)) {
      failures.push(`${relPath}: not a safe relative fixture path`);
      continue;
    }

    const expected = gitlinks.get(relPath);
    if (expected == null) {
      failures.push(`${relPath}: not a pinned gitlink`);
      continue;
    }

    const fixtureDir = path.join(root, relPath);
    if (!isNonEmptyDirectory(fixtureDir)) {
      failures.push(`${relPath}: not hydrated (expected ${expected})`);
      continue;
    }

    const actual = readHead(fixtureDir);
    if (actual == null) {
      failures.push(`${relPath}: hydrated path is not a git checkout (expected ${expected})`);
    } else if (actual !== expected) {
      failures.push(`${relPath}: checked out ${actual}, expected ${expected}`);
    }
  }
  return failures;
}

function uniqueFixturePaths(fixturePaths) {
  return [...new Set(fixturePaths)].sort((left, right) => left.localeCompare(right));
}

function isRelativeFixturePath(relPath) {
  if (path.posix.isAbsolute(relPath)) return false;
  const normalized = path.posix.normalize(relPath);
  return normalized === relPath && normalized !== "." && !normalized.startsWith("..");
}

function isNonEmptyDirectory(directory) {
  try {
    return existsSync(directory) && readdirSync(directory).length > 0;
  } catch {
    return false;
  }
}

function readGitlinks(root, fixturePaths) {
  if (fixturePaths.length === 0) return new Map();
  const result = git(root, ["ls-files", "--stage", "--", ...fixturePaths]);
  return new Map(
    result
      .split("\n")
      .map((line) => /^160000\s+([0-9a-f]{40})\s+\d+\t(.+)$/.exec(line))
      .filter((match) => match != null)
      .map((match) => [match[2], match[1]]),
  );
}

function gitlinkFixturePaths(root, fixturePaths) {
  const safePaths = fixturePaths.filter(isRelativeFixturePath);
  const gitlinks = readGitlinks(root, safePaths);
  return fixturePaths.filter((relPath) => gitlinks.has(relPath));
}

function recoveryCommandLines(recoveryPaths) {
  if (recoveryPaths.length === 0) return [];
  return [
    "Run:",
    `  git submodule update --init --depth 1 -- ${recoveryPaths.map(shellQuote).join(" ")}`,
  ];
}

function readHead(directory) {
  const result = spawnSync("git", ["-C", directory, "rev-parse", "HEAD"], {
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
  });
  if (result.status !== 0) return null;
  return result.stdout.trim();
}

function git(cwd, args) {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
  });
  if (result.status !== 0) {
    const detail = result.stderr.trim() || result.stdout.trim() || `exit ${result.status}`;
    throw new Error(`git ${args.join(" ")} failed: ${detail}`);
  }
  return result.stdout.trim();
}

function shellQuote(value) {
  if (/^[A-Za-z0-9_./-]+$/.test(value)) return value;
  return `'${value.replaceAll("'", "'\\''")}'`;
}
