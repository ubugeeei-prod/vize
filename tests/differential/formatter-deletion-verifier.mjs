import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const commitPattern = /^[0-9a-f]{40}$/;
const hashPattern = /^[0-9a-f]{64}$/;
const properties = ["idempotence", "parse-preservation", "lint-agreement"];
const requiredStages = [
  "source-cli",
  "check-write-agreement",
  "idempotence",
  "parse-preservation",
  "lint-agreement",
];

function record(value, label) {
  assert(value && typeof value === "object" && !Array.isArray(value), `${label} is required`);
  return value;
}

function integer(value, label) {
  assert(Number.isSafeInteger(value) && value >= 0, `${label} must be a non-negative integer`);
  return value;
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.keys(value)
      .sort((left, right) => left.localeCompare(right))
      .map((key) => [key, canonical(value[key])]),
  );
}

export function sha256(value) {
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(String(value));
  return createHash("sha256").update(bytes).digest("hex");
}

function canonicalSha256(value) {
  return sha256(JSON.stringify(canonical(value)));
}

function validateBuildReceipt(receipt, sourceRevision) {
  record(receipt, "formatter source-build receipt");
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.recipe, "cargo build --profile ci -p vize");
  assert.equal(receipt.sourceRevision, sourceRevision);
  assert.match(receipt.binaryPath, /^target\/ci\/vize(?:\.exe)?$/);
  assert.match(receipt.binarySha256, hashPattern);
  assert.match(receipt.cliVersion, /^vize \d+\.\d+\.\d+-dev$/);
  return canonicalSha256(receipt);
}

function validateFormatterCheck(value, label) {
  const check = record(value, label);
  assert.deepEqual(Object.keys(check).sort(), [
    "changedFileCount",
    "changedPathsSha256",
    "checkedFileCount",
    "unchangedFileCount",
  ]);
  const checked = integer(check.checkedFileCount, `${label}.checkedFileCount`);
  const changed = integer(check.changedFileCount, `${label}.changedFileCount`);
  const unchanged = integer(check.unchangedFileCount, `${label}.unchangedFileCount`);
  assert.equal(checked, changed + unchanged, `${label} counts do not reconcile`);
  assert.match(check.changedPathsSha256, hashPattern);
  return { checked, check };
}

function exactIds(actual, expected, label) {
  assert(Array.isArray(actual), `${label} must be an array`);
  assert.deepEqual(
    [...actual].sort((left, right) => left.localeCompare(right)),
    [...expected].sort((left, right) => left.localeCompare(right)),
    `${label} does not match the selected real-project shard`,
  );
}

function validatePropertyArtifact(artifact, property, sourceRevision, expectedProjects) {
  record(artifact, `${property} artifact`);
  assert.equal(artifact.schema, "vize.glyphCorpusPropertyEvidence");
  assert.equal(artifact.version, 1);
  assert.equal(artifact.property, property);
  assert.equal(artifact.sourceCommit, sourceRevision);
  exactIds(artifact.projectIds, expectedProjects, `${property}.projectIds`);
  assert(Array.isArray(artifact.violations), `${property}.violations must be an array`);
  assert(
    Array.isArray(artifact.waivedDifferences),
    `${property}.waivedDifferences must be an array`,
  );
  assert(Array.isArray(artifact.baselineUnusable), `${property}.baselineUnusable must be an array`);
  const summary = record(artifact.summary, `${property}.summary`);
  assert.equal(
    summary.waivedDifferenceCount,
    artifact.waivedDifferences.length,
    `${property}.summary.waivedDifferenceCount does not match its evidence`,
  );
  assert.equal(
    summary.baselineUnusableCount,
    artifact.baselineUnusable.length,
    `${property}.summary.baselineUnusableCount does not match its evidence`,
  );
  assert.equal(
    summary.violationCount,
    artifact.violations.length,
    `${property}.summary.violationCount does not match its evidence`,
  );
  return {
    cleanFileCount: integer(summary.cleanFileCount, `${property}.summary.cleanFileCount`),
    waivedDifferenceCount: integer(
      summary.waivedDifferenceCount,
      `${property}.summary.waivedDifferenceCount`,
    ),
    baselineUnusableCount: integer(
      summary.baselineUnusableCount,
      `${property}.summary.baselineUnusableCount`,
    ),
    violationCount: integer(summary.violationCount, `${property}.summary.violationCount`),
    waiverValidationError: summary.waiverValidationError ?? null,
    artifactSha256: canonicalSha256(artifact),
  };
}

function formatterRun(matrixProject) {
  assert(Array.isArray(matrixProject.runs), `${matrixProject.id}: runs are required`);
  const matches = matrixProject.runs.filter((run) => run.tool === "formatter");
  assert.equal(matches.length, 1, `${matrixProject.id}: exactly one formatter run is required`);
  return matches[0];
}

function validateRawFormatterRun(raw, projectId) {
  record(raw, `${projectId} formatter report`);
  assert.equal(raw.schema, "vize.fixtureToolRun");
  assert.equal(raw.version, 1);
  assert.equal(raw.project, projectId);
  assert.equal(raw.tool, "formatter");
  assert(!("spawnError" in raw), `${projectId}: formatter spawn failed`);
  assert(!("validationError" in raw), `${projectId}: formatter validation failed`);
  assert(!("parseError" in raw), `${projectId}: formatter report did not parse`);
  assert([0, 1].includes(raw.exitCode), `${projectId}: formatter exit code is invalid`);
  return validateFormatterCheck(raw.formatterCheck, `${projectId}.formatterCheck`);
}

export function assessFormatterDeletionShard({
  sourceRevision,
  registrySha256,
  shardIndex,
  shardCount,
  registryProjects,
  matrix,
  rawFormatterRuns,
  buildReceipt,
  propertyArtifacts,
}) {
  assert.match(sourceRevision, commitPattern);
  assert.match(registrySha256, hashPattern);
  integer(shardIndex, "shardIndex");
  assert(Number.isSafeInteger(shardCount) && shardCount > 0, "shardCount must be positive");
  assert(shardIndex < shardCount, "shardIndex must be less than shardCount");
  assert(Array.isArray(registryProjects), "registry projects are required");
  const selected = registryProjects.filter((_, index) => index % shardCount === shardIndex);
  assert(selected.length > 0, "formatter verifier refuses an empty shard");
  const selectedIds = selected.map((project) => project.id);
  assert.equal(new Set(selectedIds).size, selectedIds.length, "registry project IDs are duplicate");

  record(matrix, "fixture tool matrix report");
  assert.equal(matrix.schema, "vize.fixtureToolMatrixReport");
  assert.equal(matrix.version, 3);
  assert.equal(matrix.evidence?.commitSha, sourceRevision);
  assert.equal(matrix.command?.dryRun, false);
  assert.equal(matrix.command?.shardIndex, shardIndex);
  assert.equal(matrix.command?.shardCount, shardCount);
  assert(matrix.command?.tools?.includes("formatter"), "matrix did not execute formatter");
  assert(Array.isArray(matrix.projects), "matrix projects are required");
  exactIds(
    matrix.projects.map((project) => project.id),
    selectedIds,
    "matrix.projects",
  );

  const buildReceiptSha256 = validateBuildReceipt(buildReceipt, sourceRevision);
  const blockers = [];
  const projects = [];
  let checkedFileCount = 0;
  for (const registryProject of selected) {
    const matrixProject = matrix.projects.find((project) => project.id === registryProject.id);
    assert(matrixProject, `${registryProject.id}: matrix project is missing`);
    assert.equal(matrixProject.revision ?? null, registryProject.revision ?? null);
    const run = formatterRun(matrixProject);
    const raw = rawFormatterRuns.get(registryProject.id);
    const { checked, check } = validateRawFormatterRun(raw, registryProject.id);
    assert.equal(run.fileCount, checked, `${registryProject.id}: matrix file count drifted`);
    assert.equal(
      run.status,
      raw.exitCode === 0 ? "ok" : "findings",
      `${registryProject.id}: matrix formatter status drifted`,
    );
    const expectedFileCount =
      registryProject.expectedVueFileCount == null
        ? null
        : integer(
            registryProject.expectedVueFileCount,
            `${registryProject.id}.expectedVueFileCount`,
          );
    if (expectedFileCount === null ? checked === 0 : checked !== expectedFileCount) {
      blockers.push({
        code: "formatter-project-scope",
        detail:
          `${registryProject.id}: checked ${checked} file(s), expected ` +
          (expectedFileCount === null ? "a nonzero real corpus" : expectedFileCount),
      });
    }
    checkedFileCount += checked;
    projects.push({
      id: registryProject.id,
      revision: registryProject.revision ?? null,
      expectedFileCount,
      formatterCheck: check,
    });
  }

  const formatterProjects = projects.map(({ id }) => id);
  const oracles = {};
  for (const property of properties) {
    const summary = validatePropertyArtifact(
      propertyArtifacts[property],
      property,
      sourceRevision,
      formatterProjects,
    );
    oracles[property] = summary;
    if (summary.cleanFileCount !== checkedFileCount) {
      blockers.push({
        code: "formatter-file-scope",
        detail: `${property}: ${summary.cleanFileCount}/${checkedFileCount} files are clean`,
      });
    }
    if (
      summary.waivedDifferenceCount !== 0 ||
      summary.baselineUnusableCount !== 0 ||
      summary.violationCount !== 0 ||
      summary.waiverValidationError !== null
    ) {
      blockers.push({
        code: "formatter-fallback",
        detail:
          `${property}: ${summary.violationCount} violation(s), ` +
          `${summary.waivedDifferenceCount} waiver(s), ` +
          `${summary.baselineUnusableCount} unusable baseline(s)`,
      });
    }
  }

  const unsigned = {
    schema: "vize.deletionReadiness.formatterShard",
    version: 1,
    sourceRevision,
    target: "fmt",
    registry: {
      path: "tests/_fixtures/vue-ecosystem-fixtures.json",
      sha256: registrySha256,
    },
    shard: { index: shardIndex, count: shardCount },
    buildReceiptSha256,
    binarySha256: buildReceipt.binarySha256,
    requiredStages,
    projects,
    totals: { projectCount: projects.length, checkedFileCount },
    oracles,
    status: blockers.length === 0 ? "ready" : "blocked",
    blockers,
  };
  return { ...unsigned, sha256: canonicalSha256(unsigned) };
}

export function validateFormatterDeletionArtifact(value) {
  const artifact = record(value, "formatter deletion artifact");
  assert.equal(artifact.schema, "vize.deletionReadiness.formatterShard");
  assert.equal(artifact.version, 1);
  assert.match(artifact.sourceRevision, commitPattern);
  assert.equal(artifact.target, "fmt");
  assert.equal(artifact.registry?.path, "tests/_fixtures/vue-ecosystem-fixtures.json");
  assert.match(artifact.registry?.sha256, hashPattern);
  assert.match(artifact.buildReceiptSha256, hashPattern);
  assert.match(artifact.binarySha256, hashPattern);
  assert.deepEqual(artifact.requiredStages, requiredStages);
  integer(artifact.shard?.index, "formatter deletion shard index");
  assert(
    Number.isSafeInteger(artifact.shard?.count) && artifact.shard.count > artifact.shard.index,
    "formatter deletion shard count is invalid",
  );
  assert(Array.isArray(artifact.projects) && artifact.projects.length > 0);
  assert.equal(
    new Set(artifact.projects.map((project) => project.id)).size,
    artifact.projects.length,
  );
  let checkedFileCount = 0;
  for (const project of artifact.projects) {
    assert.match(project.id, /^[a-z0-9][a-z0-9-]*$/);
    assert(project.revision === null || commitPattern.test(project.revision));
    assert(
      project.expectedFileCount === null ||
        (Number.isSafeInteger(project.expectedFileCount) && project.expectedFileCount >= 0),
      `${project.id}.expectedFileCount must be null or a non-negative integer`,
    );
    const checked = validateFormatterCheck(
      project.formatterCheck,
      `${project.id}.formatterCheck`,
    ).checked;
    if (artifact.status === "ready") {
      if (project.expectedFileCount === null) assert(checked > 0);
      else assert.equal(checked, project.expectedFileCount);
    }
    checkedFileCount += checked;
  }
  assert.deepEqual(artifact.totals, {
    projectCount: artifact.projects.length,
    checkedFileCount,
  });
  assert.deepEqual(Object.keys(artifact.oracles).sort(), [...properties].sort());
  for (const property of properties) {
    const summary = record(artifact.oracles[property], `${property} oracle`);
    const cleanFileCount = integer(summary.cleanFileCount, `${property}.cleanFileCount`);
    const waivedDifferenceCount = integer(
      summary.waivedDifferenceCount,
      `${property}.waivedDifferenceCount`,
    );
    const baselineUnusableCount = integer(
      summary.baselineUnusableCount,
      `${property}.baselineUnusableCount`,
    );
    const violationCount = integer(summary.violationCount, `${property}.violationCount`);
    assert(
      summary.waiverValidationError === null || typeof summary.waiverValidationError === "string",
    );
    assert.match(summary.artifactSha256, hashPattern);
    if (artifact.status === "ready") {
      assert.equal(cleanFileCount, checkedFileCount);
      assert.equal(waivedDifferenceCount, 0);
      assert.equal(baselineUnusableCount, 0);
      assert.equal(violationCount, 0);
      assert.equal(summary.waiverValidationError, null);
    }
  }
  assert(["ready", "blocked"].includes(artifact.status));
  assert(Array.isArray(artifact.blockers));
  for (const blocker of artifact.blockers) {
    assert.equal(typeof blocker?.code, "string");
    assert(blocker.code.length > 0);
    assert.equal(typeof blocker?.detail, "string");
    assert(blocker.detail.length > 0);
  }
  assert.deepEqual(artifact.status === "ready", artifact.blockers.length === 0);
  const { sha256: recorded, ...unsigned } = artifact;
  assert.match(recorded, hashPattern);
  assert.equal(recorded, canonicalSha256(unsigned), "formatter deletion artifact digest mismatch");
  return artifact;
}

export const formatterDeletionAdapter = {
  requiredStages,
  verifyBuildReceipt(report, sourceRevision) {
    try {
      return (
        validateBuildReceipt(report.buildReceipt, sourceRevision) === report.buildReceiptSha256
      );
    } catch {
      return false;
    }
  },
  verifyObservation(observation, fixture, _row, report) {
    try {
      const artifact = validateFormatterDeletionArtifact(observation);
      return (
        artifact.status === "ready" &&
        artifact.sourceRevision === report.sourceRevision &&
        artifact.buildReceiptSha256 === report.buildReceiptSha256 &&
        typeof fixture.projectId === "string" &&
        artifact.projects.some((project) => project.id === fixture.projectId)
      );
    } catch {
      return false;
    }
  },
  verifyComparison(row) {
    try {
      const artifact = validateFormatterDeletionArtifact(row.native?.observation);
      return (
        row.comparison?.state === "equal" &&
        row.comparison?.contract === "formatter-real-project-zero-fallback-v1" &&
        row.comparison?.observationSha256 === artifact.sha256 &&
        artifact.status === "ready"
      );
    } catch {
      return false;
    }
  },
};

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function inside(directory, candidate) {
  const resolved = path.resolve(candidate);
  const relative = path.relative(path.resolve(directory), resolved);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  return resolved;
}

export function recordFormatterDeletionShard(root, reportDir) {
  const sourceRevision = process.env.GITHUB_SHA;
  assert.match(sourceRevision ?? "", commitPattern, "GITHUB_SHA must be an exact commit");
  const shardIndex = Number(process.env.FIXTURE_SHARD_INDEX);
  const shardCount = Number(process.env.FIXTURE_SHARD_COUNT);
  const registryPath = path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json");
  const registryBytes = fs.readFileSync(registryPath);
  const registry = JSON.parse(registryBytes);
  const matrix = readJson(path.join(reportDir, "summary.json"));
  const rawFormatterRuns = new Map();
  for (const project of matrix.projects ?? []) {
    const run = formatterRun(project);
    rawFormatterRuns.set(
      project.id,
      readJson(inside(reportDir, path.resolve(root, run.outputPath))),
    );
  }
  const propertyArtifacts = Object.fromEntries(
    properties.map((property) => [
      property,
      readJson(path.join(reportDir, `glyph-${property}.json`)),
    ]),
  );
  const artifact = assessFormatterDeletionShard({
    sourceRevision,
    registrySha256: sha256(registryBytes),
    shardIndex,
    shardCount,
    registryProjects: registry.projects,
    matrix,
    rawFormatterRuns,
    buildReceipt: readJson(path.join(root, "target/ci/vize.differential-build.json")),
    propertyArtifacts,
  });
  const output = path.join(reportDir, "formatter-deletion-readiness.json");
  fs.writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  process.stdout.write(
    `formatter deletion evidence: ${artifact.status}, ${artifact.totals.projectCount} project(s), ` +
      `${artifact.totals.checkedFileCount} file(s), ${artifact.blockers.length} blocker(s)\n`,
  );
  return artifact;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
  const reportDir = path.resolve(process.argv[2] ?? "");
  assert(process.argv[2], "usage: formatter-deletion-verifier.mjs REPORT_DIR");
  recordFormatterDeletionShard(root, reportDir);
}
