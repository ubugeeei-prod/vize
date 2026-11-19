import { createHash } from "node:crypto";
import { readFileSync, statSync, type Stats } from "node:fs";
import { isAbsolute, resolve } from "node:path";

import { typecheckUnits } from "./typecheck-targets.ts";

export interface TypecheckTargetProject {
  readonly id: string;
  readonly tsconfig?: unknown;
  readonly typecheckPerformance?: Readonly<Record<string, unknown>>;
  readonly [field: string]: unknown;
}

const lockfiles: Readonly<Record<string, string>> = {
  npm: "package-lock.json",
  pnpm: "pnpm-lock.yaml",
  yarn: "yarn.lock",
};

export function validateTypecheckPerformanceTarget(
  project: TypecheckTargetProject,
  fixtureRoot: string,
  { requireBaseline = false }: { readonly requireBaseline?: boolean } = {},
): void {
  const performance = project.typecheckPerformance;
  if (performance?.enabled !== true) return;
  if (performance.targets !== undefined) {
    validateTargets(project, performance);
    for (const unit of typecheckUnits(project as Parameters<typeof typecheckUnits>[0])) {
      validateTypecheckPerformanceTarget(unit as TypecheckTargetProject, fixtureRoot, {
        requireBaseline,
      });
    }
    return;
  }
  if (performance.compareTo !== "vue-tsc") {
    invalid(project, "compareTo must be vue-tsc");
  }
  requireFile(project, fixtureRoot, project.tsconfig, "tsconfig");
  // Optional: when shared `vueGlobs` reach past what `tsconfig` owns, the
  // typechecker corpus is narrowed here so one config is never asked to
  // answer for files it never included (#4454).
  const corpus = performance.corpusGlobs;
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
  const manager = performance.packageManager;
  const lockfile =
    typeof manager === "string" && Object.hasOwn(lockfiles, manager)
      ? lockfiles[manager]
      : undefined;
  if (lockfile === undefined) invalid(project, "packageManager must be npm, pnpm, or yarn");
  if (
    !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(
      typeof performance.packageManagerVersion === "string"
        ? performance.packageManagerVersion
        : "",
    )
  ) {
    invalid(project, "packageManagerVersion must be an exact semantic version");
  }
  if (performance.lockfile !== lockfile) {
    invalid(project, `lockfile must be ${lockfile} for ${String(manager)}`);
  }
  if (
    performance.nodeLinker !== undefined &&
    (manager !== "pnpm" || performance.nodeLinker !== "hoisted")
  ) {
    invalid(project, "nodeLinker must be hoisted and is only supported for pnpm");
  }
  if (
    performance.workspacePackages !== undefined &&
    (manager !== "pnpm" || performance.workspacePackages !== "injected")
  ) {
    invalid(project, "workspacePackages must be injected and is only supported for pnpm");
  }
  requireFile(project, fixtureRoot, performance.lockfile, "lockfile");
  const baseline = performance.baseline as Readonly<Record<string, unknown>> | null | undefined;
  if (baseline == null) return;
  if (
    Object.keys(baseline).some((key) => !["excludedInputs", "prepare", "tsconfig"].includes(key))
  ) {
    invalid(project, "baseline has unknown fields");
  }
  requireRelativePath(project, baseline.tsconfig, "baseline tsconfig");
  validateExcludedInputs(project, fixtureRoot, baseline.excludedInputs);
  if (baseline.prepare == null) {
    requireFile(project, fixtureRoot, baseline.tsconfig, "baseline tsconfig");
    return;
  }
  if (requireBaseline) requireFile(project, fixtureRoot, baseline.tsconfig, "baseline tsconfig");
  const commands = baselinePrepareCommands(baseline.prepare);
  if (
    commands == null ||
    commands.some(
      (command) =>
        command.length < 2 ||
        command[0] !== manager ||
        command.some(
          (argument) =>
            typeof argument !== "string" || argument.length === 0 || argument.includes("\0"),
        ),
    )
  ) {
    invalid(
      project,
      `baseline prepare must be a ${String(manager)} command argument array or a list of them`,
    );
  }
}

function validateExcludedInputs(
  project: TypecheckTargetProject,
  fixtureRoot: string,
  value: unknown,
): void {
  if (value === undefined) return;
  if (!Array.isArray(value) || value.length === 0) {
    invalid(project, "baseline.excludedInputs must be a non-empty array when present");
  }
  const seen = new Set<string>();
  for (const [index, entry] of value.entries()) {
    const label = `baseline.excludedInputs[${index}]`;
    if (
      entry == null ||
      typeof entry !== "object" ||
      Object.keys(entry).sort().join(",") !== "path,reason,sha256"
    ) {
      invalid(project, `${label} must contain exactly path, reason, and sha256`);
    }
    const record = entry as Readonly<Record<string, unknown>>;
    requireRelativePath(project, record.path, `${label}.path`);
    const inputPath = record.path as string;
    if (
      !inputPath.endsWith(".d.ts") &&
      !inputPath.endsWith(".d.mts") &&
      !inputPath.endsWith(".d.cts")
    ) {
      invalid(project, `${label}.path must be a declaration file`);
    }
    if (seen.has(inputPath)) invalid(project, `duplicate baseline excluded input ${inputPath}`);
    seen.add(inputPath);
    if (typeof record.reason !== "string" || record.reason.trim().length === 0) {
      invalid(project, `${label}.reason must be a non-empty string`);
    }
    if (typeof record.sha256 !== "string" || !/^[a-f0-9]{64}$/.test(record.sha256)) {
      invalid(project, `${label}.sha256 must be a lowercase SHA-256 digest`);
    }
    requireFile(project, fixtureRoot, inputPath, `${label}.path`);
    const actual = createHash("sha256")
      .update(readFileSync(resolve(fixtureRoot, inputPath)))
      .digest("hex");
    if (actual !== record.sha256) {
      invalid(project, `${label}.sha256 does not match ${inputPath}`);
    }
  }
}

function validateTargets(
  project: TypecheckTargetProject,
  performance: Readonly<Record<string, unknown>>,
): void {
  const targets = performance.targets;
  if (!Array.isArray(targets) || targets.length === 0) {
    invalid(project, "targets must be a non-empty array when present");
  }
  // Each target owns its corpus and config; a second project-level value
  // would be a second answer for the first target.
  if (performance.corpusGlobs !== undefined) {
    invalid(project, "corpusGlobs belongs to each target when targets are present");
  }
  const baseline = performance.baseline as Readonly<Record<string, unknown>> | undefined;
  if (baseline?.tsconfig !== undefined) {
    invalid(project, "baseline.tsconfig belongs to each target when targets are present");
  }
  const ids = new Set<string>();
  for (const [index, target] of (targets as readonly Record<string, unknown>[]).entries()) {
    const allowed = ["baselineTsconfig", "configurationOnly", "corpusGlobs", "id", "tsconfig"];
    if (
      target == null ||
      typeof target !== "object" ||
      Object.keys(target).some((key) => !allowed.includes(key))
    ) {
      invalid(project, `targets[${index}] has unknown fields`);
    }
    if (typeof target.id !== "string" || !/^[a-z0-9][a-z0-9-]*$/.test(target.id)) {
      invalid(project, `targets[${index}].id must be a lowercase slug`);
    }
    if (ids.has(target.id as string)) invalid(project, `duplicate target id ${target.id}`);
    ids.add(target.id as string);
    if (!Array.isArray(target.corpusGlobs) || target.corpusGlobs.length === 0) {
      invalid(project, `targets[${index}].corpusGlobs must be a non-empty array`);
    }
    const marker = target.configurationOnly as Readonly<Record<string, unknown>> | undefined;
    if (
      marker !== undefined &&
      (typeof marker !== "object" ||
        marker == null ||
        Object.keys(marker).sort().join(",") !== "issue,reason" ||
        typeof marker.reason !== "string" ||
        marker.reason.length === 0 ||
        !Number.isSafeInteger(marker.issue) ||
        (marker.issue as number) <= 0)
    ) {
      invalid(project, `targets[${index}].configurationOnly needs a reason and a tracking issue`);
    }
  }
  if (project.tsconfig !== (targets as readonly Record<string, unknown>[])[0]?.tsconfig) {
    invalid(project, "tsconfig must be the first target's tsconfig");
  }
}

/**
 * `baseline.prepare` is one command or an ordered list of commands, for
 * fixtures whose generated inputs come from more than one script.
 */
export function baselinePrepareCommands(prepare: unknown): unknown[][] | null {
  if (!Array.isArray(prepare) || prepare.length === 0) return null;
  if (prepare.every((command) => Array.isArray(command))) return prepare as unknown[][];
  return prepare.some((argument) => Array.isArray(argument)) ? null : [prepare];
}

function requireFile(
  project: TypecheckTargetProject,
  fixtureRoot: string,
  target: unknown,
  label: string,
): void {
  requireRelativePath(project, target, label);
  const resolved = resolve(fixtureRoot, target);
  let metadata: Stats;
  try {
    metadata = statSync(resolved);
  } catch {
    return invalid(project, `${label} does not exist: ${target}`);
  }
  if (!metadata.isFile()) invalid(project, `${label} is not a file: ${target}`);
}

function requireRelativePath(
  project: TypecheckTargetProject,
  target: unknown,
  label: string,
): asserts target is string {
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

function invalid(project: TypecheckTargetProject, message: string): never {
  throw new Error(`Invalid typecheck performance target for ${project.id}: ${message}`);
}
