import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { collectVueInputPaths } from "./tool-matrix-inputs.mjs";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const registryPath = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const knownViolationsPath = join(
  repoRoot,
  "tests",
  "_fixtures",
  "glyph-corpus-known-violations.json",
);
const workspaceRoot = join(repoRoot, "target", "vize-tests", "glyph-corpus");
const maxFilesEnv = "VIZE_GLYPH_CORPUS_MAX_FILES_PER_PROJECT";

/**
 * Registry projects the glyph property suites sweep: every project that
 * declares formatter coverage and pins at least one Vue SFC. Hydration is
 * per-project — CI lanes hydrate different subsets, so callers must filter on
 * `hydrated` and treat absent fixtures as skipped, never as failures.
 */
export function loadGlyphCorpusProjects() {
  const registry = JSON.parse(readFileSync(registryPath, "utf8"));
  return registry.projects
    .filter(
      (project) => project.coverage.includes("formatter") && project.expectedVueFileCount !== 0,
    )
    .map((project) => {
      const fixtureDir = resolve(repoRoot, project.fixturePath);
      const hydrated = existsSync(fixtureDir) && readdirSync(fixtureDir).length > 0;
      return { ...project, fixtureDir, hydrated };
    });
}

/** Collect the project's registered Vue inputs, honoring the optional cap. */
export function collectProjectVueFiles(project) {
  const files = collectVueInputPaths(project.fixtureDir, project.vueGlobs);
  const rawCap = process.env[maxFilesEnv];
  if (rawCap == null || rawCap === "") return files;
  const cap = Number(rawCap);
  if (!Number.isSafeInteger(cap) || cap < 1) {
    throw new Error(`${maxFilesEnv} must be a positive integer, received: ${rawCap}`);
  }
  return files.slice(0, cap);
}

/**
 * Resolve how to launch the vize CLI. `VIZE_TEST_BIN` (test-suite convention)
 * and `VIZE_BIN` take precedence, then staged build profiles, then a `cargo
 * run` fallback so the suite still works on a clean checkout.
 */
export function resolveGlyphLaunch() {
  const executable = process.platform === "win32" ? "vize.exe" : "vize";
  const candidates = [
    process.env.VIZE_TEST_BIN,
    process.env.VIZE_BIN,
    join(repoRoot, "target", "ci", executable),
    join(repoRoot, "target", "debug", executable),
    join(repoRoot, "target", "release", executable),
  ]
    .filter(Boolean)
    .map((candidate) => resolve(repoRoot, candidate));
  for (const candidate of candidates) {
    if (!existsSync(candidate)) continue;
    const probe = spawnSync(candidate, ["--version"], { encoding: "utf8", timeout: 30_000 });
    if (probe.status === 0) return { command: candidate, prefix: [] };
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}

/** Run a vize subcommand from `cwd`, returning the raw process result. */
export function runVize(launch, cwd, args, timeoutMs = 600_000) {
  const result = spawnSync(launch.command, [...launch.prefix, ...args], {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
  });
  if (result.error != null) {
    throw new Error(`vize ${args[0]} failed to spawn in ${cwd}: ${result.error.message}`);
  }
  return result;
}

/**
 * Copy the project's Vue inputs into an isolated workspace and format them in
 * place once with `vize fmt --write --no-config`. The callback receives the
 * workspace and helpers; the workspace is removed afterwards. Formatter
 * process failures (non-0/1 exit, `Error formatting` reports) throw — the
 * corpus is expected to format cleanly, matching the fixture tool matrix.
 */
export function withFormattedWorkspace(project, files, launch, run) {
  mkdirSync(workspaceRoot, { recursive: true });
  const workspaceDir = mkdtempSync(join(workspaceRoot, `${project.id}-`));
  try {
    for (const file of files) {
      const target = join(workspaceDir, file);
      mkdirSync(dirname(target), { recursive: true });
      cpSync(join(project.fixtureDir, file), target);
    }
    formatWorkspace(project, workspaceDir, launch);
    return run({
      workspaceDir,
      reformat: () => formatWorkspace(project, workspaceDir, launch),
    });
  } finally {
    rmSync(workspaceDir, { recursive: true, force: true });
  }
}

function formatWorkspace(project, workspaceDir, launch) {
  const result = runVize(launch, workspaceDir, [
    "fmt",
    ...project.vueGlobs,
    "--write",
    "--no-config",
  ]);
  const errored = result.stderr.split("\n").filter((line) => line.startsWith("Error formatting "));
  if ((result.status !== 0 && result.status !== 1) || errored.length > 0) {
    throw new Error(
      `vize fmt --write failed for ${project.id} (exit ${result.status}):\n` +
        `${errored.join("\n") || result.stderr.slice(0, 2000)}`,
    );
  }
  return result;
}

/** Hash every corpus file in a workspace, keyed by registry-relative path. */
export function snapshotWorkspaceFiles(workspaceDir, files) {
  const snapshot = new Map();
  for (const file of files) {
    snapshot.set(file, readFileSync(join(workspaceDir, file)));
  }
  return snapshot;
}

export function sha256(content) {
  return createHash("sha256").update(content).digest("hex");
}

/**
 * Minimal line-level diff excerpt: the first diverging line with one line of
 * context on each side, truncated so corpus files cannot flood test output.
 */
export function diffExcerpt(beforeText, afterText, beforeLabel = "before", afterLabel = "after") {
  const before = beforeText.split("\n");
  const after = afterText.split("\n");
  const limit = Math.max(before.length, after.length);
  for (let index = 0; index < limit; index += 1) {
    if (before[index] === after[index]) continue;
    const lines = [`first divergence at line ${index + 1}:`];
    for (const [label, side] of [
      [beforeLabel, before],
      [afterLabel, after],
    ]) {
      for (let offset = -1; offset <= 1; offset += 1) {
        const line = side[index + offset];
        if (line == null) continue;
        const marker = offset === 0 ? ">" : " ";
        lines.push(`  ${label} ${marker} ${truncateLine(line)}`);
      }
    }
    return lines.join("\n");
  }
  return "contents differ only in trailing bytes";
}

function truncateLine(line) {
  return line.length <= 160 ? line : `${line.slice(0, 160)}...`;
}

/**
 * Known-violation skip list: tracked defects recorded while the property
 * suite stays green. Entries pin `{ property, project, path, issue }` plus an
 * optional `rule` for lint-agreement waivers; `project` and `path` accept the
 * `"*"` wildcard so a systemic defect (e.g. a formatter/linter rule
 * disagreement) is one tracked entry instead of hundreds. The suites skip
 * matching findings and surface the skip count in their output.
 */
export function loadKnownViolations(property) {
  if (!existsSync(knownViolationsPath)) return [];
  const entries = JSON.parse(readFileSync(knownViolationsPath, "utf8"));
  if (!Array.isArray(entries)) {
    throw new Error("glyph-corpus-known-violations.json must be an array");
  }
  for (const entry of entries) {
    for (const field of ["property", "project", "path", "issue"]) {
      if (typeof entry[field] !== "string" || entry[field].length === 0) {
        throw new Error(`known violation entries require a non-empty ${field}`);
      }
    }
    if (entry.rule != null && (typeof entry.rule !== "string" || entry.rule.length === 0)) {
      throw new Error("known violation rule must be a non-empty string when present");
    }
  }
  return entries.filter((entry) => entry.property === property);
}

export function isKnownViolation(knownViolations, projectId, file, rule = null) {
  return knownViolations.some(
    (entry) =>
      (entry.project === "*" || entry.project === projectId) &&
      (entry.path === "*" || entry.path === file) &&
      (entry.rule == null ? rule == null : entry.rule === rule),
  );
}

/** Render corpus violations, capped so failures stay readable. */
export function renderViolations(property, violations, detailLimit = 8) {
  const details = violations
    .slice(0, detailLimit)
    .map((violation) => `${violation.project}: ${violation.file}\n${violation.detail}`);
  const remainder = violations.length - details.length;
  if (remainder > 0) details.push(`...and ${remainder} more ${property} violation(s)`);
  return `${violations.length} ${property} violation(s):\n\n${details.join("\n\n")}`;
}
