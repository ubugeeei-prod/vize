// Corpus baseline artifact assembly, scope proof, and drift diff (Davinci
// P0-5, TS-11). `buildArtifact`/`renderArtifact` produce the committed
// fingerprint file, `verifyScope` re-proves that a run actually covered the
// manifest, and `diffRows` reports per-surface per-project drift.
//
// Node builtins only. Every produced artifact is deterministic: stable
// sorts, no timestamps, no machine identity, no absolute paths.

import {
  EXCLUDED_FIELDS,
  HASHED_FIELDS,
  NOTES_REL,
  REGISTRY_REL,
  SCHEMA,
  SCHEMA_VERSION,
} from "./corpus-baseline-contract.mjs";
import { byKey } from "./ordering.mjs";

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

export function expectedComparisonCount(manifest, surfaces) {
  return manifest.projects.length * surfaces.length;
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
  const expectedRowCount = expectedComparisonCount(manifest, expectedSurfaces);
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
  const key = (row) => `${row.surface}\u0000${row.project}`;
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
