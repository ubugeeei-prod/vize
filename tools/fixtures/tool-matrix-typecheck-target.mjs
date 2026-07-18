import { statSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";

export function validateTypecheckPerformanceTarget(project, fixtureRoot) {
  if (project.typecheckPerformance?.enabled !== true) return;
  if (project.typecheckPerformance.compareTo !== "vue-tsc") {
    invalid(project, "compareTo must be vue-tsc");
  }
  requireFile(project, fixtureRoot, project.tsconfig, "tsconfig");
  const manager = project.typecheckPerformance.packageManager;
  const lockfiles = { pnpm: "pnpm-lock.yaml", yarn: "yarn.lock" };
  if (!Object.hasOwn(lockfiles, manager)) {
    invalid(project, "packageManager must be pnpm or yarn");
  }
  if (project.typecheckPerformance.lockfile !== lockfiles[manager]) {
    invalid(project, `lockfile must be ${lockfiles[manager]} for ${manager}`);
  }
  requireFile(project, fixtureRoot, project.typecheckPerformance.lockfile, "lockfile");
}

function requireFile(project, fixtureRoot, target, label) {
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
  const resolved = resolve(fixtureRoot, target);
  let metadata;
  try {
    metadata = statSync(resolved);
  } catch {
    invalid(project, `${label} does not exist: ${target}`);
  }
  if (!metadata.isFile()) invalid(project, `${label} is not a file: ${target}`);
}

function invalid(project, message) {
  throw new Error(`Invalid typecheck performance target for ${project.id}: ${message}`);
}
