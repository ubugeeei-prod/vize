import { statSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";

export function validateTypecheckPerformanceTarget(
  project,
  fixtureRoot,
  { requireBaseline = false } = {},
) {
  if (project.typecheckPerformance?.enabled !== true) return;
  if (project.typecheckPerformance.compareTo !== "vue-tsc") {
    invalid(project, "compareTo must be vue-tsc");
  }
  requireFile(project, fixtureRoot, project.tsconfig, "tsconfig");
  // Optional: when shared `vueGlobs` reach past what `tsconfig` owns, the
  // typechecker corpus is narrowed here so one config is never asked to
  // answer for files it never included (#4454).
  const corpus = project.typecheckPerformance.corpusGlobs;
  if (corpus !== undefined) {
    if (!Array.isArray(corpus) || corpus.length === 0) {
      invalid(project, "corpusGlobs must be a non-empty array when present");
    }
    for (const glob of corpus) {
      if (typeof glob !== "string" || glob.length === 0) {
        invalid(project, "corpusGlobs entries must be non-empty strings");
      }
      requireRelativePath(project, glob, "corpusGlobs entry");
    }
  }
  const manager = project.typecheckPerformance.packageManager;
  const lockfiles = { npm: "package-lock.json", pnpm: "pnpm-lock.yaml", yarn: "yarn.lock" };
  if (!Object.hasOwn(lockfiles, manager)) {
    invalid(project, "packageManager must be npm, pnpm, or yarn");
  }
  if (
    !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(
      project.typecheckPerformance.packageManagerVersion ?? "",
    )
  ) {
    invalid(project, "packageManagerVersion must be an exact semantic version");
  }
  if (project.typecheckPerformance.lockfile !== lockfiles[manager]) {
    invalid(project, `lockfile must be ${lockfiles[manager]} for ${manager}`);
  }
  requireFile(project, fixtureRoot, project.typecheckPerformance.lockfile, "lockfile");
  const baseline = project.typecheckPerformance.baseline;
  if (baseline == null) return;
  requireRelativePath(project, baseline.tsconfig, "baseline tsconfig");
  if (baseline.prepare == null) {
    requireFile(project, fixtureRoot, baseline.tsconfig, "baseline tsconfig");
    return;
  }
  if (requireBaseline) requireFile(project, fixtureRoot, baseline.tsconfig, "baseline tsconfig");
  if (
    !Array.isArray(baseline.prepare) ||
    baseline.prepare.length < 2 ||
    baseline.prepare[0] !== manager ||
    baseline.prepare.some(
      (argument) =>
        typeof argument !== "string" || argument.length === 0 || argument.includes("\0"),
    )
  ) {
    invalid(project, `baseline prepare must be a ${manager} command argument array`);
  }
}

function requireFile(project, fixtureRoot, target, label) {
  requireRelativePath(project, target, label);
  const resolved = resolve(fixtureRoot, target);
  let metadata;
  try {
    metadata = statSync(resolved);
  } catch {
    invalid(project, `${label} does not exist: ${target}`);
  }
  if (!metadata.isFile()) invalid(project, `${label} is not a file: ${target}`);
}

function requireRelativePath(project, target, label) {
  if (
    typeof target !== "string" ||
    target.length === 0 ||
    isAbsolute(target) ||
    /^[A-Za-z]:[\\/]/.test(target) ||
    target.includes("\\") ||
    target.startsWith("./") ||
    target.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(project, `${label} must be a normalized relative path`);
  }
}

function invalid(project, message) {
  throw new Error(`Invalid typecheck performance target for ${project.id}: ${message}`);
}
