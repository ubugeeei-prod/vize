import assert from "node:assert/strict";
import { sha256 } from "./sha256.ts";
import fs from "node:fs";
import path from "node:path";

export { sha256 } from "./sha256.ts";

export const canonicalDialects = [
  "vue0-template",
  "vue1-template",
  "vue2-sfc",
  "vue2.7-sfc",
  "vue3-sfc",
  "petite-vue",
  "pug-template",
  "jsx-babel",
  "jsx-vapor",
  "js",
  "ts",
  "jsx",
  "tsx",
  "vue-quirks",
];

const dialectCoverageStates = ["unknown", "partial", "source-inspected", "verified"];
const provisionalDialectStates = new Set(["partial", "source-inspected"]);
const dialectLabel = /^[a-z0-9][a-z0-9.-]*$/;

export function normalizeDialectCoverage(value, context) {
  const coverage = value ?? { state: "unknown", registryRef: null, evidence: [] };
  assert(
    coverage && typeof coverage === "object" && !Array.isArray(coverage),
    `${context}: coverage is required`,
  );
  assert.deepEqual(
    Object.keys(coverage).sort(),
    ["evidence", "registryRef", "state"],
    `${context}: coverage schema is closed`,
  );
  assert(dialectCoverageStates.includes(coverage.state), `${context}: invalid coverage state`);
  assert(
    coverage.registryRef === null || typeof coverage.registryRef === "string",
    `${context}: registryRef must be a string or null`,
  );
  assert(Array.isArray(coverage.evidence), `${context}: evidence must be an array`);
  for (const [index, evidence] of coverage.evidence.entries()) {
    const evidenceContext = `${context}.evidence[${index}]`;
    assert(
      evidence && typeof evidence === "object" && !Array.isArray(evidence),
      `${evidenceContext}: evidence is required`,
    );
    assert.deepEqual(
      Object.keys(evidence).sort(),
      evidence.selector === undefined ? ["dialects", "kind"] : ["dialects", "kind", "selector"],
      `${evidenceContext}: evidence schema is closed`,
    );
    assert.equal(typeof evidence.kind, "string", `${evidenceContext}: kind is required`);
    assert(
      Array.isArray(evidence.dialects) && evidence.dialects.length > 0,
      `${evidenceContext}: dialects are required`,
    );
    assert.equal(
      new Set(evidence.dialects).size,
      evidence.dialects.length,
      `${evidenceContext}: duplicate dialect`,
    );
    for (const dialect of evidence.dialects) {
      assert.match(dialect, dialectLabel, `${evidenceContext}: invalid dialect label`);
      if (coverage.state === "verified") {
        assert(
          canonicalDialects.includes(dialect),
          `${evidenceContext}: verified dialect must be canonical`,
        );
      }
    }
    if (evidence.selector !== undefined) {
      assert.equal(
        typeof evidence.selector,
        "string",
        `${evidenceContext}: selector must be a string`,
      );
      assert(evidence.selector.length > 0, `${evidenceContext}: selector cannot be empty`);
    }
  }
  if (coverage.state !== "unknown") {
    assert(coverage.evidence.length > 0, `${context}: non-unknown coverage needs evidence`);
  }
  return structuredClone(coverage);
}

export function classifyDialectCoverage(coverage) {
  const dialects = [...new Set(coverage.evidence.flatMap((item) => item.dialects))].sort();
  return {
    state: coverage.state,
    dialects,
    readiness: coverage.state === "verified" ? "verified" : "unknown",
    measurement:
      coverage.state === "verified"
        ? "verified"
        : provisionalDialectStates.has(coverage.state)
          ? "provisional"
          : "unknown",
  };
}

export function summarizeDialectCoverage(cases) {
  const summary = {
    plannedCases: cases.length,
    verifiedCases: 0,
    provisionalCases: 0,
    unknownReadinessCases: 0,
    verified: {},
    provisional: {},
  };
  for (const fixture of cases) {
    const coverage = classifyDialectCoverage(fixture.dialectCoverage);
    if (coverage.readiness === "unknown") summary.unknownReadinessCases += 1;
    if (coverage.measurement === "verified") summary.verifiedCases += 1;
    if (coverage.measurement === "provisional") summary.provisionalCases += 1;
    if (coverage.measurement === "unknown") continue;
    for (const dialect of coverage.dialects) {
      const bucket = summary[coverage.measurement];
      bucket[dialect] = (bucket[dialect] ?? 0) + 1;
    }
  }
  summary.verified = Object.fromEntries(
    Object.entries(summary.verified).sort(([a], [b]) => a.localeCompare(b)),
  );
  summary.provisional = Object.fromEntries(
    Object.entries(summary.provisional).sort(([a], [b]) => a.localeCompare(b)),
  );
  return summary;
}

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
    fixture.dialectCoverage = normalizeDialectCoverage(
      fixture.dialectCoverage,
      `${fixture.id}.dialectCoverage`,
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
  const required = new Set(requiredStages);
  assert.equal(required.size, requiredStages.length, "duplicate required product stage");
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
  if (proof.contributions.length !== required.size) return "unverified";
  const stages = new Set();
  for (const part of proof.contributions) {
    if (!required.has(part.stage) || stages.has(part.stage)) return "unverified";
    stages.add(part.stage);
    if (
      part.implementation !== "native" ||
      part.factOrigin !== "native" ||
      part.fallback !== false
    ) {
      return "legacy-backed";
    }
  }
  return verifyObservation(row.native.observation) === true ? "native-handled" : "unverified";
}
