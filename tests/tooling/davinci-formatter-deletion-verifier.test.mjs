import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  assessFormatterDeletionShard,
  formatterDeletionAdapter,
  validateFormatterDeletionArtifact,
} from "../differential/formatter-deletion-verifier.mjs";

const sourceRevision = "a".repeat(40);
const registrySha256 = "b".repeat(64);
const binarySha256 = "c".repeat(64);

function formatterCheck(checkedFileCount, changedFileCount = 0) {
  return {
    checkedFileCount,
    changedFileCount,
    unchangedFileCount: checkedFileCount - changedFileCount,
    changedPathsSha256: "d".repeat(64),
  };
}

function propertyArtifact(property, changes = {}) {
  const summary = {
    cleanFileCount: 2,
    waivedDifferenceCount: 0,
    baselineUnusableCount: 0,
    violationCount: 0,
    waiverValidationError: null,
    ...changes,
  };
  return {
    schema: "vize.glyphCorpusPropertyEvidence",
    version: 1,
    property,
    sourceCommit: sourceRevision,
    projectIds: ["with-vue", "without-vue"],
    summary,
    waivedDifferences: Array.from({ length: summary.waivedDifferenceCount }, () => ({})),
    baselineUnusable: Array.from({ length: summary.baselineUnusableCount }, () => ({})),
    violations: Array.from({ length: summary.violationCount }, () => ({})),
  };
}

function control(changes = {}) {
  const buildReceipt = {
    schema: "vize.differential.build",
    version: 1,
    recipe: "cargo build --profile ci -p vize",
    sourceRevision,
    binaryPath: "target/ci/vize",
    binarySha256,
    cliVersion: "vize 0.500.0-dev",
  };
  const registryProjects = [
    { id: "with-vue", revision: "e".repeat(40) },
    { id: "without-vue", revision: "f".repeat(40), expectedVueFileCount: 0 },
  ];
  const matrix = {
    schema: "vize.fixtureToolMatrixReport",
    version: 3,
    evidence: { commitSha: sourceRevision },
    command: { dryRun: false, shardIndex: 0, shardCount: 1, tools: ["formatter"] },
    projects: registryProjects.map((project) => ({
      id: project.id,
      revision: project.revision,
      runs: [
        {
          tool: "formatter",
          status: "findings",
          fileCount: project.id === "with-vue" ? 2 : 0,
        },
      ],
    })),
  };
  const rawFormatterRuns = new Map(
    registryProjects.map((project) => [
      project.id,
      {
        schema: "vize.fixtureToolRun",
        version: 1,
        project: project.id,
        tool: "formatter",
        exitCode: 1,
        formatterCheck: formatterCheck(
          project.id === "with-vue" ? 2 : 0,
          project.id === "with-vue" ? 1 : 0,
        ),
      },
    ]),
  );
  const propertyArtifacts = Object.fromEntries(
    ["idempotence", "parse-preservation", "lint-agreement"].map((property) => [
      property,
      propertyArtifact(property),
    ]),
  );
  return {
    sourceRevision,
    registrySha256,
    shardIndex: 0,
    shardCount: 1,
    registryProjects,
    matrix,
    rawFormatterRuns,
    buildReceipt,
    propertyArtifacts,
    ...changes,
  };
}

void test("formatter verifier binds the source CLI to every selected real project and file", () => {
  const artifact = assessFormatterDeletionShard(control());
  assert.equal(artifact.status, "ready");
  assert.deepEqual(artifact.blockers, []);
  assert.deepEqual(artifact.totals, { projectCount: 2, checkedFileCount: 2 });
  assert.equal(
    artifact.projects.find((project) => project.id === "with-vue").expectedFileCount,
    null,
  );
  assert.equal(
    artifact.projects.find((project) => project.id === "without-vue").expectedFileCount,
    0,
  );
  assert.doesNotThrow(() => validateFormatterDeletionArtifact(artifact));

  const report = {
    sourceRevision,
    buildReceipt: control().buildReceipt,
    buildReceiptSha256: artifact.buildReceiptSha256,
  };
  const fixture = { projectId: "with-vue" };
  const row = {
    native: { observation: artifact },
    comparison: {
      state: "equal",
      contract: "formatter-real-project-zero-fallback-v1",
      observationSha256: artifact.sha256,
    },
  };
  assert.equal(formatterDeletionAdapter.verifyBuildReceipt(report, sourceRevision), true);
  assert.equal(formatterDeletionAdapter.verifyObservation(artifact, fixture, row, report), true);
  assert.equal(formatterDeletionAdapter.verifyComparison(row, fixture, report), true);
});

void test("formatter verifier records genuine corpus gaps but never grants readiness credit", () => {
  for (const summaryChanges of [
    { violationCount: 1 },
    { waivedDifferenceCount: 1 },
    { baselineUnusableCount: 1 },
    { cleanFileCount: 1 },
    { waiverValidationError: "unconsumed waiver" },
  ]) {
    const input = control();
    input.propertyArtifacts["parse-preservation"] = propertyArtifact(
      "parse-preservation",
      summaryChanges,
    );
    const artifact = assessFormatterDeletionShard(input);
    assert.equal(artifact.status, "blocked");
    assert(artifact.blockers.some((blocker) => blocker.code.startsWith("formatter-")));
    assert.equal(
      formatterDeletionAdapter.verifyObservation(
        artifact,
        { projectId: "with-vue" },
        { native: { observation: artifact } },
        { sourceRevision, buildReceiptSha256: artifact.buildReceiptSha256 },
      ),
      false,
    );
  }
});

void test("formatter verifier rejects reduced shards, forged reports and stale identity", () => {
  const missing = control();
  missing.matrix.projects.pop();
  assert.throws(() => assessFormatterDeletionShard(missing), /matrix\.projects/);

  const forged = control();
  forged.rawFormatterRuns.get("with-vue").formatterCheck.checkedFileCount = 1;
  assert.throws(() => assessFormatterDeletionShard(forged), /counts do not reconcile/);

  const stale = control();
  stale.propertyArtifacts.idempotence.sourceCommit = "0".repeat(40);
  assert.throws(() => assessFormatterDeletionShard(stale));

  const falseSummary = control();
  falseSummary.propertyArtifacts.idempotence.violations.push({ file: "src/App.vue" });
  assert.throws(() => assessFormatterDeletionShard(falseSummary), /does not match its evidence/);

  const reduced = control();
  reduced.rawFormatterRuns.set("with-vue", {
    ...reduced.rawFormatterRuns.get("with-vue"),
    formatterCheck: formatterCheck(0),
  });
  reduced.matrix.projects[0].runs[0].fileCount = 0;
  const reducedArtifact = assessFormatterDeletionShard(reduced);
  assert.equal(reducedArtifact.status, "blocked");
  assert(reducedArtifact.blockers.some((blocker) => blocker.code === "formatter-project-scope"));

  const artifact = assessFormatterDeletionShard(control());
  artifact.projects[0].formatterCheck.checkedFileCount = 1;
  assert.throws(() => validateFormatterDeletionArtifact(artifact));
});

void test("the real-project matrix records the formatter verifier artifact on every shard", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/real-project-matrix.yml", import.meta.url),
    "utf8",
  );
  assert.match(workflow, /- name: Record formatter deletion-readiness evidence/);
  assert.match(
    workflow,
    /node tests\/differential\/formatter-deletion-verifier\.mjs "\$FIXTURE_REPORT_DIR"/,
  );
  assert.match(workflow, /formatter-deletion-readiness\.json/);
});
