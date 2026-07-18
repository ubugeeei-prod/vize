import { statSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";

export function validateTypecheckPerformanceTarget(project, fixtureRoot) {
  if (project.typecheckPerformance?.enabled !== true) return;
  if (project.typecheckPerformance.compareTo !== "vue-tsc") {
    invalid(project, "compareTo must be vue-tsc");
  }
  const target = project.tsconfig;
  if (
    typeof target !== "string" ||
    target.length === 0 ||
    isAbsolute(target) ||
    /^[A-Za-z]:[\\/]/.test(target) ||
    target.includes("\\") ||
    target.startsWith("./") ||
    target.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(project, "tsconfig must be a normalized relative path");
  }
  const resolved = resolve(fixtureRoot, target);
  let metadata;
  try {
    metadata = statSync(resolved);
  } catch {
    invalid(project, `tsconfig does not exist: ${target}`);
  }
  if (!metadata.isFile()) invalid(project, `tsconfig is not a file: ${target}`);
}

function invalid(project, message) {
  throw new Error(`Invalid typecheck performance target for ${project.id}: ${message}`);
}
