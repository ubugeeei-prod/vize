// Shared machinery for the Davinci corpus baseline snapshot and diff gate
// (P0-5, TS-11). `corpus-baseline.mjs` and `corpus-diff.mjs` both run the
// existing real-project harness (`tools/fixtures/tool-matrix-report.mjs`)
// across all shards, then reduce every per-project per-surface payload to a
// `{surface, project, file_count, content_hash}` row.
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
// Node builtins only. Every produced artifact is deterministic: stable
// sorts, no timestamps, no machine identity, no absolute paths.

import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
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
export const MATRIX_SCRIPT = path.join(repoRoot, "tools", "fixtures", "tool-matrix-report.mjs");

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

const PAYLOAD_FAILURE_FIELDS = ["spawnError", "parseError", "validationError"];

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
 * Run the matrix harness as `shards` parallel processes (shards are serial
 * internally) and return the shard output directories. Throws unless every
 * shard exits 0 with a fully successful summary.
 */
export async function runMatrix({ shards, vizeBin, tools, scratchDir, timeoutMs, log }) {
  mkdirSync(scratchDir, { recursive: true });
  const runs = [];
  for (let index = 0; index < shards; index += 1) {
    const outputDir = path.join(scratchDir, `shard-${index}`);
    const args = [
      MATRIX_SCRIPT,
      "--shard-index",
      String(index),
      "--shard-count",
      String(shards),
      "--vize-bin",
      vizeBin,
      "--output-dir",
      outputDir,
    ];
    if (timeoutMs != null) args.push("--timeout-ms", String(timeoutMs));
    for (const tool of tools) args.push("--tool", tool);
    runs.push({ index, outputDir, promise: spawnShard(args, index, log) });
  }
  const failures = [];
  for (const run of runs) {
    const result = await run.promise;
    if (result.code !== 0) {
      failures.push(
        `shard ${run.index} exited ${result.code}${describeShardFailure(run.outputDir)}`,
      );
    }
  }
  if (failures.length > 0) {
    throw new Error(`matrix run failed:\n  ${failures.join("\n  ")}`);
  }
  return runs.map((run) => run.outputDir);
}

function spawnShard(args, index, log) {
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(process.execPath, args, {
      cwd: repoRoot,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stderrTail = "";
    child.stdout.on("data", (chunk) => log(`[shard ${index}] ${String(chunk).trimEnd()}`));
    child.stderr.on("data", (chunk) => {
      stderrTail = `${stderrTail}${chunk}`.slice(-4000);
    });
    child.on("error", rejectPromise);
    child.on("close", (code) => {
      if (code !== 0 && stderrTail.trim().length > 0) {
        log(`[shard ${index}] stderr: ${stderrTail.trim()}`);
      }
      resolvePromise({ code });
    });
  });
}

function describeShardFailure(outputDir) {
  const summaryPath = path.join(outputDir, "summary.json");
  if (!existsSync(summaryPath)) return " (no summary.json written)";
  const summary = JSON.parse(readFileSync(summaryPath, "utf8"));
  const failed = [];
  for (const project of summary.projects) {
    for (const run of project.runs) {
      if (run.status === "failed" || run.status === "missing-fixture") {
        failed.push(`${project.id}/${run.tool}: ${run.status}`);
      }
    }
  }
  return failed.length === 0 ? "" : ` (${failed.join(", ")})`;
}

/**
 * Reduce every shard's per-run payload files to sorted
 * `{surface, project, file_count, content_hash}` rows.
 */
export function reduceShards(shardDirs, tools) {
  const rows = [];
  const seenProjects = new Set();
  for (const shardDir of shardDirs) {
    const summaryPath = path.join(shardDir, "summary.json");
    if (!existsSync(summaryPath)) {
      throw new Error(`shard output has no summary.json: ${shardDir}`);
    }
    const summary = JSON.parse(readFileSync(summaryPath, "utf8"));
    assertCleanSummary(summary, shardDir);
    for (const project of summary.projects) {
      if (seenProjects.has(project.id)) {
        throw new Error(`project ${project.id} appears in more than one shard`);
      }
      seenProjects.add(project.id);
      const runTools = project.runs.map((run) => run.tool).sort(byKey);
      const expectedTools = [...tools].sort(byKey);
      if (JSON.stringify(runTools) !== JSON.stringify(expectedTools)) {
        throw new Error(
          `project ${project.id} ran surfaces [${runTools.join(", ")}], expected [${expectedTools.join(", ")}]`,
        );
      }
      for (const run of project.runs) {
        rows.push(reduceRun(shardDir, project.id, run));
      }
    }
  }
  rows.sort(
    (left, right) => byKey(left.surface, right.surface) || byKey(left.project, right.project),
  );
  return rows;
}

function assertCleanSummary(summary, shardDir) {
  const counts = summary.summary;
  const clean =
    counts.failedRuns === 0 &&
    counts.missingFixtureRuns === 0 &&
    counts.plannedRuns === 0 &&
    counts.okRuns + counts.findingsRuns === counts.runCount;
  if (!clean) {
    throw new Error(`shard summary is not clean: ${shardDir} (${JSON.stringify(counts)})`);
  }
}

function reduceRun(shardDir, projectId, run) {
  const payloadPath = path.join(shardDir, `${projectId}-${run.tool}.json`);
  if (!existsSync(payloadPath)) {
    throw new Error(`run payload is missing: ${payloadPath}`);
  }
  const payload = JSON.parse(readFileSync(payloadPath, "utf8"));
  for (const field of PAYLOAD_FAILURE_FIELDS) {
    if (payload[field] != null) {
      throw new Error(`${projectId}/${run.tool} payload carries ${field}: ${payload[field]}`);
    }
  }
  const hashedFields = HASHED_FIELDS[run.tool];
  if (hashedFields == null) throw new Error(`unsupported surface: ${run.tool}`);
  const content = {};
  for (const field of hashedFields) {
    if (!(field in payload)) {
      throw new Error(`${projectId}/${run.tool} payload has no ${field} field`);
    }
    content[field] = payload[field];
  }
  const fileCount = payloadFileCount(run.tool, payload);
  if (run.fileCount !== fileCount) {
    throw new Error(
      `${projectId}/${run.tool} summary fileCount ${run.fileCount} != payload-derived ${fileCount}`,
    );
  }
  return {
    surface: run.tool,
    project: projectId,
    file_count: fileCount,
    content_hash: sha256(canonicalJson(content)),
  };
}

/** Mirrors tools/fixtures/tool-matrix-metrics.mjs, from the payload side. */
function payloadFileCount(tool, payload) {
  if (tool === "compiler") return payload.compilerArtifacts.inputFileCount;
  if (tool === "typechecker") return payload.parsed.fileCount;
  if (tool === "linter") return payload.parsed.length;
  if (tool === "formatter") return payload.formatterCheck.checkedFileCount;
  throw new Error(`unsupported surface: ${tool}`);
}

export function sha256(text) {
  return createHash("sha256").update(text).digest("hex");
}

/** JSON with every object's keys sorted, so hashes ignore insertion order. */
export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    const keys = Object.keys(value).sort(byKey);
    return `{${keys.map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

/** Assemble the committed artifact with a fixed key order. */
export function buildArtifact(rows, manifest) {
  const projects = [...new Set(rows.map((row) => row.project))].sort(byKey);
  const surfaces = [...new Set(rows.map((row) => row.surface))].sort(byKey);
  const fileCountBySurface = {};
  for (const surface of surfaces) fileCountBySurface[surface] = 0;
  let totalFileCount = 0;
  for (const row of rows) {
    fileCountBySurface[row.surface] += row.file_count;
    totalFileCount += row.file_count;
  }
  const artifact = {
    schema: SCHEMA,
    version: SCHEMA_VERSION,
    registry: REGISTRY_REL,
    notes: NOTES_REL,
    hashed_fields: HASHED_FIELDS,
    excluded_fields: EXCLUDED_FIELDS,
    scope: {
      manifest_project_count: manifest.projects.length,
      projects_run: projects.length,
      surfaces,
      surfaces_per_project: surfaces.length,
      row_count: rows.length,
      total_file_count: totalFileCount,
      file_count_by_surface: fileCountBySurface,
    },
    rows,
  };
  return artifact;
}

export function renderArtifact(artifact) {
  return `${JSON.stringify(artifact, null, 2)}\n`;
}

/**
 * Scope proof (TS-11): the artifact must cover every manifest project on
 * every requested surface, and must not be a zero-file run. Returns a list
 * of exact failure reasons; empty means the proof holds.
 */
export function verifyScope(artifact, manifest, surfaces, label) {
  const reasons = [];
  const manifestIds = manifest.projects.map((project) => project.id).sort(byKey);
  const expectedSurfaces = [...surfaces].sort(byKey);
  const scope = artifact.scope ?? {};
  if (artifact.schema !== SCHEMA || artifact.version !== SCHEMA_VERSION) {
    reasons.push(`${label}: schema is not ${SCHEMA} v${SCHEMA_VERSION}`);
    return reasons;
  }
  if (scope.manifest_project_count !== manifestIds.length) {
    reasons.push(
      `${label}: scope.manifest_project_count ${scope.manifest_project_count} != manifest ${manifestIds.length}`,
    );
  }
  if (JSON.stringify(scope.surfaces) !== JSON.stringify(expectedSurfaces)) {
    reasons.push(
      `${label}: scope.surfaces [${(scope.surfaces ?? []).join(", ")}] != expected [${expectedSurfaces.join(", ")}]`,
    );
  }
  const rows = Array.isArray(artifact.rows) ? artifact.rows : [];
  if (scope.row_count !== rows.length) {
    reasons.push(`${label}: scope.row_count ${scope.row_count} != ${rows.length} rows`);
  }
  const expectedRowCount = manifestIds.length * expectedSurfaces.length;
  if (rows.length !== expectedRowCount) {
    reasons.push(
      `${label}: ${rows.length} rows != ${manifestIds.length} projects x ${expectedSurfaces.length} surfaces = ${expectedRowCount}`,
    );
  }
  for (const surface of expectedSurfaces) {
    const surfaceProjects = rows
      .filter((row) => row.surface === surface)
      .map((row) => row.project)
      .sort(byKey);
    const missing = manifestIds.filter((id) => !surfaceProjects.includes(id));
    const extra = surfaceProjects.filter((id) => !manifestIds.includes(id));
    if (missing.length > 0) {
      reasons.push(`${label}: surface ${surface} is missing projects [${missing.join(", ")}]`);
    }
    if (extra.length > 0) {
      reasons.push(`${label}: surface ${surface} has unknown projects [${extra.join(", ")}]`);
    }
  }
  let totalFileCount = 0;
  for (const row of rows) {
    if (!Number.isSafeInteger(row.file_count) || row.file_count < 0) {
      reasons.push(`${label}: ${row.surface}/${row.project} has invalid file_count`);
      continue;
    }
    totalFileCount += row.file_count;
  }
  if (scope.total_file_count !== totalFileCount) {
    reasons.push(
      `${label}: scope.total_file_count ${scope.total_file_count} != ${totalFileCount} summed`,
    );
  }
  if (totalFileCount === 0) {
    reasons.push(`${label}: zero-file run (total_file_count is 0)`);
  }
  const declaredZero = new Set(
    manifest.projects
      .filter((project) => project.expectedVueFileCount === 0)
      .map((project) => project.id),
  );
  for (const row of rows) {
    if (row.file_count === 0 && !declaredZero.has(row.project)) {
      reasons.push(
        `${label}: ${row.surface}/${row.project} ran zero files but the manifest does not declare expectedVueFileCount 0`,
      );
    }
  }
  return reasons;
}

/** Compare two row sets; returns sorted drift records. */
export function diffRows(baselineRows, freshRows) {
  const key = (row) => `${row.surface} ${row.project}`;
  const baselineByKey = new Map(baselineRows.map((row) => [key(row), row]));
  const freshByKey = new Map(freshRows.map((row) => [key(row), row]));
  const drift = [];
  for (const [rowKey, baselineRow] of baselineByKey) {
    const freshRow = freshByKey.get(rowKey);
    if (freshRow == null) {
      drift.push({ ...baselineRow, kind: "missing-in-fresh" });
    } else if (
      freshRow.content_hash !== baselineRow.content_hash ||
      freshRow.file_count !== baselineRow.file_count
    ) {
      drift.push({
        surface: baselineRow.surface,
        project: baselineRow.project,
        kind: "changed",
        baseline_file_count: baselineRow.file_count,
        fresh_file_count: freshRow.file_count,
        baseline_hash: baselineRow.content_hash,
        fresh_hash: freshRow.content_hash,
      });
    }
  }
  for (const [rowKey, freshRow] of freshByKey) {
    if (!baselineByKey.has(rowKey)) drift.push({ ...freshRow, kind: "missing-in-baseline" });
  }
  drift.sort(
    (left, right) => byKey(left.surface, right.surface) || byKey(left.project, right.project),
  );
  return drift;
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

export function resolveVizeBin(vizeBin) {
  const resolved = path.resolve(repoRoot, vizeBin ?? path.join("target", "release", "vize"));
  if (!existsSync(resolved)) {
    throw new Error(
      `vize binary not found: ${resolved} (build with: cargo build --release -p vize)`,
    );
  }
  return resolved;
}

export function scratchRoot(label) {
  return path.join(repoRoot, ".vize", "davinci-corpus", label);
}

export function cleanupScratch(dir) {
  rmSync(dir, { recursive: true, force: true });
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
