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
// davinci-road/plan/corpus-baseline-notes.md): the sha256 of a
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

import { byKey } from "./ordering.mjs";
import { repoRoot } from "./paths.mjs";

export const SCHEMA = "vize.davinciCorpusBaseline";
export const SCHEMA_VERSION = 1;
export const UNSTABLE_SCHEMA = "vize.davinciCorpusUnstableRows";
export const REGISTRY_REL = "tests/_fixtures/vue-ecosystem-fixtures.json";
export const BASELINE_REL = "tests/_fixtures/davinci-baseline.json";
export const NOTES_REL = "davinci-road/plan/corpus-baseline-notes.md";
export const UNSTABLE_REL = "davinci-road/plan/corpus-baseline-unstable.json";
export const BASELINE_PATH = path.join(repoRoot, BASELINE_REL);
export const UNSTABLE_PATH = path.join(repoRoot, UNSTABLE_REL);

/** The harness tool lanes, in the artifact's canonical (sorted) order. */
export const SURFACES = ["compiler", "formatter", "linter", "typechecker"];

/** Payload fields whose canonical JSON forms each surface's content hash. */
export const HASHED_FIELDS = {
  compiler: ["compilerArtifacts", "exitCode", "stdout"],
  formatter: ["exitCode", "formatterCheck", "stdout"],
  linter: ["exitCode", "stderr", "stdout"],
  typechecker: ["exitCode", "stderr", "stdout", "typecheckerCoverage"],
};

/** Fields deliberately left out of the hash, with the reason on record. */
export const EXCLUDED_FIELDS = {
  compiler: ["stderr"],
  formatter: ["stderr"],
};

export function loadManifest() {
  const registryPath = path.join(repoRoot, REGISTRY_REL);
  const registry = JSON.parse(readFileSync(registryPath, "utf8"));
  if (!Array.isArray(registry.projects) || registry.projects.length === 0) {
    throw new Error(`${REGISTRY_REL} lists no projects`);
  }
  const ids = registry.projects.map((project) => project.id);
  if (new Set(ids).size !== ids.length) {
    throw new Error(`${REGISTRY_REL} contains duplicate project ids`);
  }
  return registry;
}

/**
 * Load the filed-nondeterminism sidecar (P0-5 "shard-scoped" rows). Rows
 * listed there still appear in the baseline and in drift reports, but
 * their drift does not gate. Missing sidecar means no unstable rows.
 * Every entry must name a known surface, a manifest project, and a
 * non-empty reason — a stale or typo'd allowlist is an error, not a
 * silent no-op.
 */
export function loadUnstableRows(manifest) {
  if (!existsSync(UNSTABLE_PATH)) return [];
  const sidecar = JSON.parse(readFileSync(UNSTABLE_PATH, "utf8"));
  if (sidecar.schema !== UNSTABLE_SCHEMA || sidecar.version !== 1) {
    throw new Error(`${UNSTABLE_REL}: schema is not ${UNSTABLE_SCHEMA} v1`);
  }
  if (!Array.isArray(sidecar.rows)) throw new Error(`${UNSTABLE_REL}: rows must be an array`);
  const manifestIds = new Set(manifest.projects.map((project) => project.id));
  const seen = new Set();
  for (const row of sidecar.rows) {
    if (!SURFACES.includes(row.surface)) {
      throw new Error(`${UNSTABLE_REL}: unknown surface ${row.surface}`);
    }
    if (!manifestIds.has(row.project)) {
      throw new Error(`${UNSTABLE_REL}: unknown project ${row.project}`);
    }
    if (typeof row.reason !== "string" || row.reason.length === 0) {
      throw new Error(`${UNSTABLE_REL}: ${row.surface}/${row.project} has no reason`);
    }
    const key = `${row.surface} ${row.project}`;
    if (seen.has(key)) throw new Error(`${UNSTABLE_REL}: duplicate row ${key}`);
    seen.add(key);
  }
  return sidecar.rows;
}

export function parseSurfaceFilter(values) {
  const surfaces = [...new Set(values)].sort(byKey);
  for (const surface of surfaces) {
    if (!SURFACES.includes(surface)) {
      throw new Error(`unknown surface: ${surface} (expected one of ${SURFACES.join(", ")})`);
    }
  }
  return surfaces.length === 0 ? SURFACES : surfaces;
}
