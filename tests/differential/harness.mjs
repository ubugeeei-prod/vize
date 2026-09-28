import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function readPinnedArtifact(root, file) {
  assert.equal(typeof file.path, "string", "artifact path is required");
  assert(!path.isAbsolute(file.path), "artifact path must be relative");
  const resolved = fs.realpathSync(path.resolve(root, file.path));
  const relative = path.relative(fs.realpathSync(root), resolved);
  assert(
    relative !== "" && !relative.startsWith("..") && !path.isAbsolute(relative),
    "artifact must remain inside its fixture root",
  );
  assert(fs.statSync(resolved).isFile(), "artifact must be a file");
  assert.match(file.sha256, /^[a-f0-9]{64}$/, "artifact sha256 is required");
  const bytes = fs.readFileSync(resolved);
  assert.equal(sha256(bytes), file.sha256, `immutable artifact SHA256 mismatch: ${file.path}`);
  return bytes;
}

// Product adapters validate their own options, artifacts, observations and comparator.
// This shared envelope fixes planned case identity before either lane can run.
export function loadProductManifest(manifestPath, product) {
  assert.match(product, /^[a-z][a-z0-9-]*$/);
  const raw = fs.readFileSync(manifestPath);
  const manifest = JSON.parse(raw.toString("utf8"));
  assert.equal(manifest.schema, "vize.differential.manifest");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.product, product, "no registered adapter for this product");
  assert.match(manifest.baseRevision, /^[a-f0-9]{40}$/);
  assert(Array.isArray(manifest.cases) && manifest.cases.length > 0, "planned cases are required");
  const ids = new Set();
  for (const fixture of manifest.cases) {
    assert.equal(typeof fixture.id, "string");
    assert.match(
      fixture.id,
      new RegExp(`^${product}/[a-z0-9/-]+$`),
      `wrong product case: ${fixture.id}`,
    );
    assert(!ids.has(fixture.id), `duplicate planned case: ${fixture.id}`);
    ids.add(fixture.id);
    assert(["draft", "active"].includes(fixture.state));
    assert(Array.isArray(fixture.targets) && fixture.targets.length > 0);
    for (const target of fixture.targets) assert.match(target, /^[a-z][a-z0-9_-]*$/);
    assert.equal(
      new Set(fixture.targets).size,
      fixture.targets.length,
      `duplicate target: ${fixture.id}`,
    );
  }
  return { manifest, cases: manifest.cases, manifestSha256: sha256(raw) };
}

export function runPlannedCases(loaded, adapter) {
  assert.equal(typeof adapter.runCase, "function", "product runCase adapter is required");
  const rows = loaded.cases.flatMap((fixture) => adapter.runCase(fixture));
  assertExactRows(loaded, rows);
  return rows;
}

export function assertExactRows(loaded, rows) {
  assert(Array.isArray(rows));
  const planned = new Map(loaded.cases.map((fixture) => [fixture.id, fixture.targets]));
  const plannedCount = loaded.cases.reduce((count, fixture) => count + fixture.targets.length, 0);
  assert.equal(rows.length, plannedCount, "missing or extra planned result rows");
  const observed = new Set();
  for (const row of rows) {
    assert.equal(typeof row?.id, "string", "result case ID is required");
    assert(planned.has(row.id), `unplanned result row: ${row.id}`);
    const targets = planned.get(row.id);
    const target = row.target ?? (targets.length === 1 ? targets[0] : null);
    assert(targets.includes(target), `unplanned result target: ${row.id}/${target}`);
    const key = JSON.stringify([row.id, target]);
    assert(!observed.has(key), `duplicate result row: ${row.id}/${target}`);
    observed.add(key);
    assert(
      row.legacy && row.native && row.comparison,
      `both lanes and comparison required: ${row.id}`,
    );
  }
}

export function validateResultEnvelope(loaded, report, sourceRevision) {
  assert.equal(report.schema, "vize.differential.result");
  assert.equal(report.version, 1);
  assert.equal(report.product, loaded.manifest.product);
  assert.equal(report.manifestSha256, loaded.manifestSha256);
  assert.equal(report.sourceRevision, sourceRevision, "unexpected actual source revision");
  assertExactRows(loaded, report.rows);
  assert.equal(report.summary?.plannedCases, loaded.cases.length);
}

// The adapter must verify an actual native observation before credit is possible.
// Backends built on old product facts remain legacy-backed for whole-product rates.
export function classifyNativeRow(
  row,
  { sourceRevision, buildReceiptSha256, requiredStages, verifyObservation },
) {
  if (row.native?.state !== "completed") return row.native?.state ?? "unverified";
  assert(Array.isArray(requiredStages) && requiredStages.length > 0);
  assert.equal(typeof verifyObservation, "function", "native observation verifier is required");
  const proof = row.native.provenance;
  if (
    proof?.scope !== "whole-product" ||
    proof.sourceRevision !== sourceRevision ||
    proof.buildReceiptSha256 !== buildReceiptSha256 ||
    !/^[a-f0-9]{64}$/.test(buildReceiptSha256)
  ) {
    return "unverified";
  }
  if (!Array.isArray(proof.contributions)) return "unverified";
  const stages = new Set();
  for (const part of proof.contributions) {
    if (typeof part.stage !== "string" || stages.has(part.stage)) return "unverified";
    stages.add(part.stage);
    if (
      part.implementation !== "native" ||
      part.factOrigin !== "native" ||
      part.fallback !== false
    ) {
      return "legacy-backed";
    }
  }
  if (requiredStages.some((stage) => !stages.has(stage))) return "unverified";
  return verifyObservation(row.native.observation) === true ? "native-handled" : "unverified";
}
