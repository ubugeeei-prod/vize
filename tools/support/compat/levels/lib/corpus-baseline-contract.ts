// Corpus baseline contract (Davinci P0-5, TS-11): the artifact schema, the
// surface list, the per-surface hash contract, and the two inputs both tools
// read — the fixture manifest and the filed-nondeterminism sidecar.
//
// Surface list: exactly the tool lanes the harness emits today — `compiler`
// (`vize build --format json`, the single DOM-backend compile lane; the
// harness has no separate vapor/ssr lanes), `typechecker` (`vize check
// --format json`), `linter` (`vize lint --format json --preset ecosystem`),
// and `formatter` (`vize fmt --check`).
//
// Hash contract per surface (documented in
// docs/davinci/plan/corpus-baseline-notes.md): the sha256 of a
// key-sorted canonical JSON of the fields listed in `HASHED_FIELDS`.
// Two fields are excluded as filed nondeterminism, verified empirically
// by back-to-back runs: the compiler lane's `stderr` (absolute mkdtemp
// output paths in its `Built:` lines, a wall-clock banner, load-dependent
// slow-file warnings, rayon-ordered error listings) and the formatter
// lane's `stderr` (`Would reformat:` lines print in rayon
// thread-completion order). Their deterministic evidence is hashed
// instead: `compilerArtifacts` (byte digest of every compiled artifact)
// and `formatterCheck` (counts + sorted changed-path digest).
//
// Node builtins only.

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

import { asRecord, isRecord } from "../../../json-value.ts";
import type { JsonRecord } from "../../../json-value.ts";
import { byKey } from "./ordering.ts";
import { repoRoot } from "./paths.ts";

export interface CorpusProject {
  id: string;
  expectedVueFileCount?: unknown;
}

export interface CorpusManifest {
  projects: readonly CorpusProject[];
}

export interface UnstableRow {
  surface: string;
  project: string;
  reason: string;
}

function isCorpusProject(value: unknown): value is CorpusProject & JsonRecord {
  return isRecord(value) && typeof value.id === "string";
}

export const SCHEMA = "vize.davinciCorpusBaseline";
export const SCHEMA_VERSION = 1;
export const UNSTABLE_SCHEMA = "vize.davinciCorpusUnstableRows";
export const REGISTRY_REL = "tests/_fixtures/vue-ecosystem-fixtures.json";
export const BASELINE_REL = "tests/_fixtures/davinci-baseline.json";
export const NOTES_REL = "docs/davinci/plan/corpus-baseline-notes.md";
export const UNSTABLE_REL = "docs/davinci/plan/corpus-baseline-unstable.json";
export const BASELINE_PATH = path.join(repoRoot, BASELINE_REL);
export const UNSTABLE_PATH = path.join(repoRoot, UNSTABLE_REL);

/** The harness tool lanes, in the artifact's canonical (sorted) order. */
export const SURFACES = ["compiler", "formatter", "linter", "typechecker"];

/** Payload fields whose canonical JSON forms each surface's content hash. */
export const HASHED_FIELDS: Record<string, readonly string[]> = {
  compiler: ["compilerArtifacts", "exitCode", "stdout"],
  formatter: ["exitCode", "formatterCheck", "stdout"],
  linter: ["exitCode", "stderr", "stdout"],
  typechecker: ["exitCode", "stderr", "stdout", "typecheckerCoverage"],
};

/** Fields deliberately left out of the hash, with the reason on record. */
export const EXCLUDED_FIELDS: Record<string, readonly string[]> = {
  compiler: ["stderr"],
  formatter: ["stderr"],
};

export function loadManifest(): JsonRecord & { projects: (CorpusProject & JsonRecord)[] } {
  const registryPath = path.join(repoRoot, REGISTRY_REL);
  const registry = asRecord(JSON.parse(readFileSync(registryPath, "utf8")), REGISTRY_REL);
  const projects: unknown = registry.projects;
  if (!Array.isArray(projects) || projects.length === 0) {
    throw new Error(`${REGISTRY_REL} lists no projects`);
  }
  if (!projects.every(isCorpusProject)) {
    throw new Error(`${REGISTRY_REL} lists a project without a string id`);
  }
  const ids = projects.map((project) => project.id);
  if (new Set(ids).size !== ids.length) {
    throw new Error(`${REGISTRY_REL} contains duplicate project ids`);
  }
  return { ...registry, projects };
}

/**
 * Load the filed-nondeterminism sidecar (P0-5 "shard-scoped" rows). Rows
 * listed there still appear in the baseline and in drift reports, but
 * their drift does not gate. Missing sidecar means no unstable rows.
 * Every entry must name a known surface, a manifest project, and a
 * non-empty reason — a stale or typo'd allowlist is an error, not a
 * silent no-op.
 */
export function loadUnstableRows(manifest: CorpusManifest): UnstableRow[] {
  if (!existsSync(UNSTABLE_PATH)) return [];
  const sidecar = asRecord(JSON.parse(readFileSync(UNSTABLE_PATH, "utf8")), UNSTABLE_REL);
  if (sidecar.schema !== UNSTABLE_SCHEMA || sidecar.version !== 1) {
    throw new Error(`${UNSTABLE_REL}: schema is not ${UNSTABLE_SCHEMA} v1`);
  }
  const rows: unknown = sidecar.rows;
  if (!Array.isArray(rows)) throw new Error(`${UNSTABLE_REL}: rows must be an array`);
  const manifestIds = new Set(manifest.projects.map((project) => project.id));
  const seen = new Set<string>();
  const typed: UnstableRow[] = [];
  for (const value of rows) {
    const row = asRecord(value, `${UNSTABLE_REL} row`);
    const { surface, project, reason } = row;
    if (typeof surface !== "string" || !SURFACES.includes(surface)) {
      throw new Error(`${UNSTABLE_REL}: unknown surface ${String(surface)}`);
    }
    if (typeof project !== "string" || !manifestIds.has(project)) {
      throw new Error(`${UNSTABLE_REL}: unknown project ${String(project)}`);
    }
    if (typeof reason !== "string" || reason.length === 0) {
      throw new Error(`${UNSTABLE_REL}: ${surface}/${project} has no reason`);
    }
    const key = `${surface} ${project}`;
    if (seen.has(key)) throw new Error(`${UNSTABLE_REL}: duplicate row ${key}`);
    seen.add(key);
    typed.push({ ...row, surface, project, reason });
  }
  return typed;
}

export function parseSurfaceFilter(values: readonly string[]): string[] {
  const surfaces = [...new Set(values)].sort(byKey);
  for (const surface of surfaces) {
    if (!SURFACES.includes(surface)) {
      throw new Error(`unknown surface: ${surface} (expected one of ${SURFACES.join(", ")})`);
    }
  }
  return surfaces.length === 0 ? SURFACES : surfaces;
}
