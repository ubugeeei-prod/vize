import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  createFormatterChangeEvidence,
  validateFormatterChangeEvidence,
} from "./tool-matrix-formatter.ts";
import { collectVueInputPaths } from "./tool-matrix-inputs.ts";
import {
  readKnownViolationLedger,
  type GlyphProperty,
  type KnownViolation,
} from "./glyph-corpus-waivers.ts";
import type { FormatterChangeEvidence } from "./tool-matrix-formatter.ts";

export {
  assertHydratedKnownViolationPaths,
  auditKnownViolationIssues,
  createKnownViolationConsumption,
  validateKnownViolationEntries,
  writeGlyphCorpusPropertyEvidence,
} from "./glyph-corpus-waivers.ts";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "..");
const registryPath = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const knownViolationsPath = join(
  repoRoot,
  "tests",
  "_fixtures",
  "glyph-corpus-known-violations.json",
);
const workspaceRoot = join(repoRoot, "target", "vize-tests", "glyph-corpus");
const maxFilesEnv = "VIZE_GLYPH_CORPUS_MAX_FILES_PER_PROJECT";

export interface RegistryProject {
  readonly id: string;
  readonly fixturePath: string;
  readonly coverage: string[];
  readonly expectedVueFileCount?: number;
  readonly vueGlobs: string[];
  readonly revision?: string;
  readonly [field: string]: unknown;
}

export interface CorpusWorkspaceProject {
  readonly id: string;
  readonly fixtureDir: string;
  readonly vueGlobs: readonly string[];
}

export interface GlyphCorpusProject extends RegistryProject {
  readonly fixtureDir: string;
  readonly hydrated: boolean;
}

export interface GlyphLaunch {
  readonly command: string;
  readonly prefix: string[];
  readonly evidenceCommand: string;
}

type Environment = Readonly<Record<string, string | undefined>>;

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function registryProject(value: unknown): RegistryProject {
  const project = value as Partial<Record<string, unknown>> | null;
  if (
    project == null ||
    typeof project !== "object" ||
    typeof project.id !== "string" ||
    typeof project.fixturePath !== "string" ||
    !isStringArray(project.coverage) ||
    (project.expectedVueFileCount !== undefined &&
      typeof project.expectedVueFileCount !== "number") ||
    !isStringArray(project.vueGlobs)
  )
    throw new Error("invalid Vue ecosystem fixture registry project");
  return project as RegistryProject;
}

/**
 * Registry projects the glyph property suites sweep: every project that
 * declares formatter coverage and pins at least one Vue SFC. Hydration is
 * per-project. Matrix lanes pass `FIXTURE_SHARD_INDEX`/`FIXTURE_SHARD_COUNT`,
 * so shared fixture paths never make sibling registry ids look selected.
 */
export function loadGlyphCorpusProjects(): GlyphCorpusProject[] {
  return materializeGlyphCorpusProjects(selectGlyphCorpusProjects(readGlyphCorpusRegistry()));
}

function loadAllGlyphCorpusProjects(): GlyphCorpusProject[] {
  return materializeGlyphCorpusProjects(selectGlyphCorpusProjects(readGlyphCorpusRegistry(), {}));
}

function readGlyphCorpusRegistry(): RegistryProject[] {
  const registry = JSON.parse(readFileSync(registryPath, "utf8")) as { projects?: unknown };
  if (!Array.isArray(registry.projects)) throw new Error("fixture registry has no projects");
  return registry.projects.map(registryProject);
}

function materializeGlyphCorpusProjects(
  projects: readonly RegistryProject[],
): GlyphCorpusProject[] {
  return projects.map((project) => {
    const fixtureDir = resolve(repoRoot, project.fixturePath);
    const hydrated = existsSync(fixtureDir) && readdirSync(fixtureDir).length > 0;
    return { ...project, fixtureDir, hydrated };
  });
}

export function selectGlyphCorpusProjects<
  Project extends { readonly coverage: readonly string[]; readonly expectedVueFileCount?: number },
>(projects: readonly Project[], environment: Environment = process.env): Project[] {
  const shardIndex = environment.FIXTURE_SHARD_INDEX;
  const shardCount = environment.FIXTURE_SHARD_COUNT;
  if ((shardIndex == null) !== (shardCount == null)) {
    throw new Error("FIXTURE_SHARD_INDEX and FIXTURE_SHARD_COUNT must be set together");
  }
  const selected =
    shardIndex == null || shardCount == null
      ? projects
      : selectGlyphShard(projects, shardIndex, shardCount);
  return selected.filter(
    (project) => project.coverage.includes("formatter") && project.expectedVueFileCount !== 0,
  );
}

function selectGlyphShard<Project>(
  projects: readonly Project[],
  shardIndex: string,
  shardCount: string,
): Project[] {
  const index = nonNegativeInteger(shardIndex, "FIXTURE_SHARD_INDEX");
  const count = positiveInteger(shardCount, "FIXTURE_SHARD_COUNT");
  if (index >= count) throw new Error("FIXTURE_SHARD_INDEX must be less than FIXTURE_SHARD_COUNT");
  return projects.filter((_, projectIndex) => projectIndex % count === index);
}

/** Collect the project's registered Vue inputs, honoring the optional cap. */
export function collectProjectVueFiles(project: CorpusWorkspaceProject): string[] {
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
export function resolveGlyphLaunch(): GlyphLaunch {
  const executable = process.platform === "win32" ? "vize.exe" : "vize";
  const candidates = [
    process.env.VIZE_TEST_BIN,
    process.env.VIZE_BIN,
    join(repoRoot, "target", "ci", executable),
    join(repoRoot, "target", "debug", executable),
    join(repoRoot, "target", "release", executable),
  ]
    .filter((candidate): candidate is string => Boolean(candidate))
    .map((candidate) => resolve(repoRoot, candidate));
  for (const candidate of candidates) {
    if (!existsSync(candidate)) continue;
    const probe = spawnSync(candidate, ["--version"], { encoding: "utf8", timeout: 30_000 });
    if (probe.status === 0) return { command: candidate, prefix: [], evidenceCommand: candidate };
  }
  const metadata = spawnSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
    cwd: repoRoot,
    encoding: "utf8",
    timeout: 30_000,
  });
  if (metadata.status !== 0) {
    throw new Error(`cargo metadata failed while resolving formatter evidence: ${metadata.stderr}`);
  }
  const evidenceCommand = cargoEvidenceCommand(metadata.stdout);
  return {
    command: "cargo",
    prefix: ["run", "-q", "-p", "vize", "--"],
    evidenceCommand,
  };
}

export function cargoEvidenceCommand(
  metadataJson: string,
  platform: NodeJS.Platform = process.platform,
): string {
  const metadata = JSON.parse(metadataJson) as { target_directory?: unknown } | null;
  const targetDirectory = metadata?.target_directory;
  if (typeof targetDirectory !== "string" || targetDirectory === "") {
    throw new Error("cargo metadata omitted target_directory for formatter evidence");
  }
  return join(targetDirectory, "debug", platform === "win32" ? "vize.exe" : "vize");
}

/** Run a vize subcommand from `cwd`, returning the raw process result. */
export function runVize(
  launch: Pick<GlyphLaunch, "command" | "prefix">,
  cwd: string,
  args: readonly string[],
  timeoutMs = 600_000,
): SpawnSyncReturns<string> {
  const result = spawnSync(launch.command, [...launch.prefix, ...args], {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
  });
  if (result.error != null) {
    throw new Error(`vize ${args[0] ?? ""} failed to spawn in ${cwd}: ${result.error.message}`);
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
export interface FormattedWorkspace {
  readonly workspaceDir: string;
  readonly reformat: () => SpawnSyncReturns<string>;
}

export function withFormattedWorkspace<Result>(
  project: CorpusWorkspaceProject,
  files: readonly string[],
  launch: Pick<GlyphLaunch, "command" | "prefix">,
  run: (workspace: FormattedWorkspace) => Result,
): Result {
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

/** Load the preceding real-project matrix's validated formatter check evidence. */
export function loadFormatterCheckEvidence(
  project: { readonly id: string },
  reportDir = process.env.FIXTURE_REPORT_DIR,
): FormatterChangeEvidence | null {
  if (reportDir == null || reportDir === "") return null;

  const reportPath = resolve(repoRoot, reportDir, `${project.id}-formatter.json`);
  if (!existsSync(reportPath)) {
    throw new Error(`missing formatter check report for ${project.id}: ${reportPath}`);
  }
  const report = JSON.parse(readFileSync(reportPath, "utf8")) as Partial<Record<string, unknown>>;
  if (
    report.schema !== "vize.fixtureToolRun" ||
    report.version !== 1 ||
    report.project !== project.id ||
    report.tool !== "formatter"
  ) {
    throw new Error(`invalid formatter check report identity for ${project.id}: ${reportPath}`);
  }
  const evidence = report.formatterCheck;
  validateFormatterChangeEvidence(evidence, `${project.id} formatter --check evidence`);
  return evidence;
}

/** Measure which copied fixture inputs the first formatter write actually changed. */
export function collectFormatterWriteEvidence(
  project: Pick<CorpusWorkspaceProject, "fixtureDir">,
  files: readonly string[],
  workspaceDir: string,
): FormatterChangeEvidence {
  const changedPaths = files.filter(
    (file) =>
      !readFileSync(join(project.fixtureDir, file)).equals(readFileSync(join(workspaceDir, file))),
  );
  return createFormatterChangeEvidence(files.length, changedPaths);
}

/** Require formatter --check's prediction to equal the first --write mutation set. */
export function assertFormatterCheckWriteAgreement(
  projectId: string,
  checkEvidence: unknown,
  writeEvidence: unknown,
): void {
  validateFormatterChangeEvidence(checkEvidence, `${projectId} formatter --check evidence`);
  validateFormatterChangeEvidence(writeEvidence, `${projectId} formatter --write evidence`);
  const fields = [
    "checkedFileCount",
    "changedFileCount",
    "unchangedFileCount",
    "changedPathsSha256",
  ] as const;
  const mismatches = fields.filter((field) => checkEvidence[field] !== writeEvidence[field]);
  if (mismatches.length === 0) return;
  throw new Error(
    `${projectId}: formatter --check prediction disagrees with actual --write mutation ` +
      `(${mismatches.join(", ")})\n` +
      `  --check: ${JSON.stringify(checkEvidence)}\n` +
      `  --write: ${JSON.stringify(writeEvidence)}`,
  );
}

function formatWorkspace(
  project: CorpusWorkspaceProject,
  workspaceDir: string,
  launch: Pick<GlyphLaunch, "command" | "prefix">,
): SpawnSyncReturns<string> {
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
export function snapshotWorkspaceFiles(
  workspaceDir: string,
  files: readonly string[],
): Map<string, Buffer> {
  const snapshot = new Map<string, Buffer>();
  for (const file of files) {
    snapshot.set(file, readFileSync(join(workspaceDir, file)));
  }
  return snapshot;
}

export function sha256(content: string | Uint8Array): string {
  return createHash("sha256").update(content).digest("hex");
}

/**
 * Minimal line-level diff excerpt: the first diverging line with one line of
 * context on each side, truncated so corpus files cannot flood test output.
 */
export function diffExcerpt(
  beforeText: string,
  afterText: string,
  beforeLabel = "before",
  afterLabel = "after",
): string {
  const before = beforeText.split("\n");
  const after = afterText.split("\n");
  const limit = Math.max(before.length, after.length);
  for (let index = 0; index < limit; index += 1) {
    if (before[index] === after[index]) continue;
    const lines = [`first divergence at line ${index + 1}:`];
    const sides: readonly (readonly [string, readonly string[]])[] = [
      [beforeLabel, before],
      [afterLabel, after],
    ];
    for (const [label, side] of sides) {
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

function truncateLine(line: string): string {
  return line.length <= 160 ? line : `${line.slice(0, 160)}...`;
}

function positiveInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return parsed;
}

function nonNegativeInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  return parsed;
}

export function loadKnownViolationLedger(): KnownViolation[] {
  return readKnownViolationLedger(knownViolationsPath, loadAllGlyphCorpusProjects());
}

/** Load the checked-in waiver ledger entries for one corpus property. */
export function loadKnownViolations(property: GlyphProperty): KnownViolation[] {
  return loadKnownViolationLedger().filter((entry) => entry.property === property);
}

/** Publish per-file Pug semantic-oracle provenance for a hydrated matrix shard. */
export interface PugOracleFile {
  readonly project: string;
  readonly path: string;
  readonly verdict: "clean" | "violation" | "baseline-unusable";
  readonly waived?: boolean;
  readonly [field: string]: unknown;
}

export function writeGlyphPugSemanticEvidence(
  {
    projectIds,
    baseline,
    files,
  }: {
    readonly projectIds: Iterable<string>;
    readonly baseline: unknown;
    readonly files: readonly PugOracleFile[];
  },
  reportDir = process.env.FIXTURE_REPORT_DIR,
): string | null {
  if (reportDir == null || reportDir === "") return null;
  // Code-point ordering, not localeCompare: the artifact must be byte-identical
  // across runners regardless of the host's ICU locale data.
  const byCodePoint = (left: string, right: string): number =>
    left < right ? -1 : left > right ? 1 : 0;
  const sortedFiles = [...files].sort(
    (left, right) => byCodePoint(left.project, right.project) || byCodePoint(left.path, right.path),
  );
  const identities = new Set<string>();
  for (const file of sortedFiles) {
    const identity = `${file.project}\0${file.path}`;
    if (identities.has(identity)) throw new Error(`duplicate Pug oracle evidence: ${identity}`);
    identities.add(identity);
  }
  const artifact = {
    schema: "vize.glyphPugSemanticEvidence",
    version: 1,
    sourceCommit: process.env.GITHUB_SHA ?? null,
    projectIds: [...projectIds].sort(byCodePoint),
    baseline,
    // Enforced regressions, unusable baselines, and accepted waivers are three
    // distinct outcomes; collapsing them would hide why a shard is not clean.
    summary: {
      evaluatedPugFileCount: sortedFiles.length,
      cleanFileCount: sortedFiles.filter((file) => file.verdict === "clean" && !file.waived).length,
      violationCount: sortedFiles.filter((file) => file.verdict === "violation" && !file.waived)
        .length,
      baselineUnusableCount: sortedFiles.filter(
        (file) => file.verdict === "baseline-unusable" && !file.waived,
      ).length,
      waivedCount: sortedFiles.filter((file) => file.waived === true).length,
    },
    files: sortedFiles,
  };
  const output = resolve(reportDir, "glyph-pug-semantics.json");
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  return output;
}

/** Track exact waiver consumption so a fixed path makes its old waiver fail. */
/** Render corpus violations, capped so failures stay readable. */
export interface CorpusViolation {
  readonly project: string;
  readonly file: string;
  readonly detail: string;
}

export function renderViolations(
  property: string,
  violations: readonly CorpusViolation[],
  detailLimit = 8,
): string {
  const details = violations
    .slice(0, detailLimit)
    .map((violation) => `${violation.project}: ${violation.file}\n${violation.detail}`);
  const remainder = violations.length - details.length;
  if (remainder > 0) details.push(`...and ${remainder} more ${property} violation(s)`);
  return `${violations.length} ${property} violation(s):\n\n${details.join("\n\n")}`;
}
