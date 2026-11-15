import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  buildV1SupportMatrix,
  renderV1SupportMatrix,
  V1_CASES,
  V1_TOPOLOGIES,
} from "../../tools/support/compat/v1/support-matrix.ts";

const root = path.resolve(import.meta.dirname, "../..");
const output = path.join(root, "docs/release/v1-supported-matrix.json");
const matrix = buildV1SupportMatrix(root);

void test("the committed v1 support manifest is generated from the audited inventories", () => {
  assert.equal(readFileSync(output, "utf8"), renderV1SupportMatrix(root));
  assert.equal(matrix.schema, "vize.v1-qualification-manifest.v1");
  assert.equal(matrix.counts.declaredCombinations, matrix.combinations.length);
  assert.equal(matrix.counts.caseObligations, matrix.combinations.length * V1_CASES.length);
  assert.equal(
    matrix.counts.hostRuntimeCombinations,
    matrix.environments.hostRuntimeCombinations.length,
  );
  assert.equal(matrix.counts.requiredRows, matrix.requiredRows.length);
  assert.equal(matrix.counts.completeRequiredRows, 0);
  assert.equal(matrix.counts.blockingGaps, matrix.knownGaps.length);
  assert.equal(matrix.counts.packageManagers, 5);
  assert.equal(matrix.counts.freshProjectCells, 11);
  assert.equal(matrix.counts.editors, 6);
  assert.equal(matrix.counts.topologies, 6);
  assert.equal(matrix.releaseReady, false);
});

void test("every supported language and product combination has an owner and the five v1 cases", () => {
  const ids = new Set<string>();
  const surfaces = new Set<string>();
  for (const combination of matrix.combinations) {
    assert.ok(!ids.has(combination.id), `duplicate combination ${combination.id}`);
    ids.add(combination.id);
    surfaces.add(combination.surface);
    assert.match(combination.owner, /^[a-z0-9@/_-]+$/u);
    assert.deepEqual(combination.requiredCases, V1_CASES, combination.id);
    assert.ok(combination.requiredQualificationSets.length > 0, combination.id);
    assert.ok(combination.topologies.length > 0, combination.id);
    assert.ok(combination.currentTier.length > 0, combination.id);
    assert.equal(combination.v1Target, "required", combination.id);
    assert.ok(combination.qualifiedBaseline.length > 0, combination.id);
    assert.equal(combination.qualification, "unbound", combination.id);
    for (const evidence of combination.evidence) {
      assert.ok(existsSync(path.join(root, evidence)), `${combination.id}: missing ${evidence}`);
    }
  }
  assert.deepEqual([...surfaces].toSorted(), [
    "compiler",
    "integration",
    "linter",
    "lsp-editor",
    "typechecker",
  ]);
});

void test("every required product by Vue by host by runtime row fails closed", () => {
  const ids = new Set<string>();
  const products = new Set(
    matrix.combinations.map(({ product, vue, backend }) => `${product}/${vue}/${backend}`),
  );
  const hosts = new Set(
    matrix.environments.hostRuntimeCombinations.map(
      ({ runtime, version, target }) => `${runtime}/${version}/${target}`,
    ),
  );
  assert.equal(matrix.requiredRows.length, products.size * hosts.size);
  for (const row of matrix.requiredRows) {
    assert.ok(!ids.has(row.id), `duplicate required row ${row.id}`);
    ids.add(row.id);
    assert.ok(["yes", "conditional"].includes(row.supported), row.id);
    assert.ok(Array.isArray(row.positive), row.id);
    assert.ok(Array.isArray(row.negative), row.id);
    assert.ok(Array.isArray(row.manual), row.id);
    assert.equal(row.candidateSource, null, row.id);
    assert.equal(row.candidateRun, null, row.id);
    assert.deepEqual(row.evidence, [], row.id);
    assert.deepEqual(row.positive, [], row.id);
    assert.deepEqual(row.negative, [], row.id);
    assert.deepEqual(row.manual, [], row.id);
    assert.equal(row.qualification, "gap", row.id);
    assert.match(row.gap, /No exact-candidate receipt/u, row.id);
  }
});

void test("historical workflow evidence is exact but never grants candidate qualification", () => {
  assert.ok(matrix.evidenceSnapshots.length > 0);
  const snapshotIds = new Set<string>();
  for (const snapshot of matrix.evidenceSnapshots) {
    assert.ok(!snapshotIds.has(snapshot.id), `duplicate evidence snapshot ${snapshot.id}`);
    snapshotIds.add(snapshot.id);
    assert.equal(snapshot.classification, "historical-manual-context", snapshot.id);
    assert.match(snapshot.source, /^[0-9a-f]{40}$/u, snapshot.id);
    assert.ok(Number.isSafeInteger(snapshot.run), snapshot.id);
    assert.match(snapshot.url, new RegExp(`/actions/runs/${snapshot.run}$`, "u"), snapshot.id);
    assert.equal(snapshot.conclusion, "success", snapshot.id);
  }
  const credited = new Set(
    matrix.requiredRows.flatMap(({ positive, negative, manual }) => [
      ...positive,
      ...negative,
      ...manual,
    ]),
  );
  for (const snapshot of matrix.evidenceSnapshots)
    assert.ok(!credited.has(snapshot.id), snapshot.id);
});

void test("every blocking gap has one owner and a machine-checkable exit contract", () => {
  const ids = new Set<string>();
  for (const gap of matrix.knownGaps) {
    assert.ok(!ids.has(gap.id), `duplicate gap ${gap.id}`);
    ids.add(gap.id);
    assert.equal(gap.state, "blocking", gap.id);
    assert.match(gap.owner, /^[a-z0-9@/_-]+$/u, gap.id);
    assert.ok(gap.issues.length > 0, gap.id);
    assert.ok(
      gap.issues.every((issue) => Number.isSafeInteger(issue) && issue > 0),
      gap.id,
    );
    assert.ok(gap.summary.length > 0, gap.id);
    assert.ok(gap.exitCondition.length > 0, gap.id);
  }
});

void test("Vapor and legacy support remain explicit boundaries", () => {
  for (const combination of matrix.combinations) {
    if (combination.backend === "vapor") assert.equal(combination.vue, "3.6+-vapor");
    if (combination.vue === "2.7") assert.notEqual(combination.backend, "vapor");
  }
  assert.deepEqual(
    matrix.unsupported.map(({ combination, disposition }) => [combination, disposition]),
    [
      ["vue-2.7/vapor", "unsupported"],
      ["linux-musl/bun/content-mapper", "unsupported"],
      ["linux-musl/node-22", "unsupported"],
      ["wsl2/shipped-editor-hosts", "unsupported"],
      ["vue-pre-2.7/v1", "outside-v1"],
      ["eslint-prettier-babel", "outside-v1"],
    ],
  );
});

void test("all shipped targets, managers, editors and topology claims stay closed", () => {
  const environments = matrix.environments;
  assert.deepEqual(environments.nativeTargets, environments.releasedNativeTargets);
  assert.deepEqual(environments.nativeTargets, environments.vscodeProviderTargets);
  assert.equal(environments.releasedCliTargets.length, 8);
  assert.deepEqual(
    environments.packageManagers.map(({ id }) => id),
    ["bun", "npm", "pnpm", "vp", "yarn"],
  );
  assert.deepEqual(
    Object.fromEntries(
      environments.packageManagers.map(({ id, qualifiedVersion }) => [id, qualifiedVersion]),
    ),
    {
      bun: "bun@1.3.14",
      npm: "bundled-with-qualified-node",
      pnpm: "pnpm@11.21.0",
      vp: "setup-vp-v1-action",
      yarn: "yarn@4.9.2",
    },
  );
  assert.deepEqual(
    environments.editors.map(({ id }) => id),
    ["emacs", "helix", "neovim", "vim", "vscode", "zed"],
  );
  assert.deepEqual(
    matrix.topologies.map(({ id }) => id),
    V1_TOPOLOGIES,
  );
  for (const topology of matrix.topologies) {
    assert.equal(topology.qualification, "unbound", topology.id);
    assert.ok(topology.evidenceCandidates.length > 0, topology.id);
    for (const evidence of topology.evidenceCandidates)
      assert.ok(existsSync(path.join(root, evidence)), `missing topology evidence ${evidence}`);
  }
});

void test("Bun is admitted on native glibc, macOS and Windows but never silently on musl", () => {
  const bunHosts = matrix.environments.hostRuntimeCombinations.filter(
    ({ runtime }) => runtime === "bun",
  );
  assert.equal(bunHosts.length, 6);
  assert.ok(bunHosts.every(({ version }) => version === matrix.versions.bun));
  assert.ok(bunHosts.every(({ target }) => !target.endsWith("-musl")));
  assert.ok(
    matrix.environments.hostRuntimeCombinations.some(
      ({ target }) => target === "wsl2-ubuntu-24.04-x64",
    ),
  );
  const muslNode = matrix.environments.hostRuntimeCombinations.filter(
    ({ runtime, target }) => runtime === "node" && target.endsWith("-musl"),
  );
  assert.deepEqual(
    muslNode.map(({ version }) => version),
    ["24", "24"],
  );
});

void test("qualification receipts require exact source, fixture, dependency, runtime and binary identity", () => {
  assert.deepEqual(Object.keys(matrix.identityContracts), [
    "source",
    "fixture",
    "dependency",
    "runtime",
    "binary",
    "producers",
  ]);
  for (const producer of matrix.identityContracts.producers)
    assert.ok(existsSync(path.join(root, producer)), `missing identity producer ${producer}`);
});
