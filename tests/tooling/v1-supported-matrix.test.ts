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
import { CONTENT_MAPPER_PROTOCOL_FEATURES } from "../../tools/support/compat/v1/content-mapper-qualification.ts";

const root = path.resolve(import.meta.dirname, "../..");
const output = path.join(root, "docs/release/v1-supported-matrix.json");
const matrix = buildV1SupportMatrix(root);

void test("the committed v1 support manifest is generated from the audited inventories", async () => {
  assert.equal(readFileSync(output, "utf8"), await renderV1SupportMatrix(root));
  assert.equal(matrix.schema, "vize.v1-qualification-manifest.v1");
  assert.equal(matrix.counts.declaredCombinations, matrix.combinations.length);
  assert.equal(matrix.counts.caseObligations, matrix.combinations.length * V1_CASES.length);
  assert.equal(matrix.counts.caseSelectors, matrix.counts.caseObligations);
  assert.equal(matrix.caseSelectors.length, matrix.counts.caseObligations);
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

void test("the Content Mapper v1 obligation inventory is complete, owned and fails closed", () => {
  const qualification = matrix.contentMapper;
  assert.equal(qualification.schema, "vize.content-mapper-v1-qualification.v1");
  assert.equal(qualification.parentIssue, 13018);
  assert.equal(qualification.qualificationIssue, 13071);
  assert.equal(qualification.qualification, "gap");
  assert.match(qualification.pinnedTypeScriptRevision, /^[0-9a-f]{40}$/u);
  assert.equal(qualification.counts.semantic, 16);
  assert.equal(qualification.counts.protocolFeatures, 20);
  assert.equal(qualification.counts.diagnostics, 4);
  assert.equal(qualification.counts.lifecycle, 1);
  assert.equal(qualification.counts.performance, 1);
  assert.equal(qualification.counts.runtimeAndCorpus, 2);
  assert.equal(qualification.counts.unbound, 44);

  const obligations = [
    ...qualification.semantic,
    ...qualification.protocolFeatures,
    ...qualification.diagnostics,
    ...qualification.lifecycle,
    ...qualification.performance,
    ...qualification.runtimeAndCorpus,
  ];
  const ids = new Set<string>();
  for (const obligation of obligations) {
    assert.ok(!ids.has(obligation.id), `duplicate Content Mapper obligation ${obligation.id}`);
    ids.add(obligation.id);
    assert.deepEqual(obligation.requiredCases, V1_CASES, obligation.id);
    assert.equal(obligation.candidateSource, null, obligation.id);
    assert.equal(obligation.candidateRun, null, obligation.id);
    assert.deepEqual(obligation.receipts, [], obligation.id);
    assert.equal(obligation.qualification, "unbound", obligation.id);
    assert.ok(obligation.issues.length > 0, obligation.id);
    assert.ok(obligation.evidenceCandidates.length > 0, obligation.id);
    for (const evidence of obligation.evidenceCandidates)
      assert.ok(existsSync(path.join(root, evidence)), `${obligation.id}: missing ${evidence}`);
  }

  const performance = qualification.performance[0];
  assert.deepEqual(performance.measurements, [
    "cold",
    "warm",
    "no-op",
    "leaf-edit",
    "dependency-edit",
    "p95",
    "rss",
  ]);
  assert.deepEqual(performance.scales, [1_000, 5_000, 20_000]);
  assert.equal(performance.packetIdentity, "required");
  assert.equal(performance.baseHeadComparison, "required");
  assert.deepEqual(qualification.lifecycle[0].operations, [
    "open",
    "edit",
    "save",
    "close",
    "create",
    "rename",
    "delete",
  ]);
  assert.deepEqual(qualification.upstreamBlockers, [
    {
      id: "grouped-disjoint-expect-usage",
      issue: 6258,
      state: "blocked-upstream",
    },
  ]);
  assert.ok(qualification.producerProtocol.evidenceCandidates.length > 0);
  for (const evidence of qualification.producerProtocol.evidenceCandidates)
    assert.ok(existsSync(path.join(root, evidence)), `producer protocol: missing ${evidence}`);
});

void test("Content Mapper protocol features match the pinned Rust protocol contract", () => {
  assert.deepEqual(
    matrix.contentMapper.protocolFeatures.map(({ id }) => id),
    CONTENT_MAPPER_PROTOCOL_FEATURES,
  );
  const source = readFileSync(
    path.join(root, "crates/vize_canon/src/batch/virtual_project/content_mapper/span_features.rs"),
    "utf8",
  );
  const enumBody = /enum ContentMapperSpanFeature \{(?<body>[\s\S]*?)\n\}/u.exec(source)?.groups
    ?.body;
  assert.ok(enumBody, "missing ContentMapperSpanFeature enum");
  const rustFeatures = [...enumBody.matchAll(/^\s{4}([A-Za-z]+) = 1 << \d+,$/gmu)].map(
    ([, name]) => name,
  );
  assert.deepEqual(rustFeatures, CONTENT_MAPPER_PROTOCOL_FEATURES);
});

void test("Content Mapper interop examples preserve the v1 support boundary", () => {
  assert.deepEqual(
    matrix.contentMapper.boundaryExamples.map(({ id, disposition }) => [id, disposition]),
    [
      ["tsrx", "outside-v1"],
      ["svelte", "outside-v1"],
      ["astro", "outside-v1"],
    ],
  );
  for (const boundary of matrix.contentMapper.boundaryExamples) {
    assert.ok(boundary.reason.length > 0, boundary.id);
    assert.ok(boundary.evidenceCandidates.length > 0, boundary.id);
    for (const evidence of boundary.evidenceCandidates)
      assert.ok(existsSync(path.join(root, evidence)), `${boundary.id}: missing ${evidence}`);
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
  assert.deepEqual(
    Object.keys(matrix.caseReceiptIdentityContracts).toSorted(),
    [
      "source",
      "candidate",
      "actions",
      "fixture",
      "dependency",
      "runtime",
      "binary",
      "packet",
      "receipt",
    ].toSorted(),
  );
  for (const side of ["source", "candidate"] as const)
    assert.deepEqual(
      matrix.caseReceiptIdentityContracts.packet[side].toSorted(),
      [
        "diagnosticsSha256",
        "authoredRangesSha256",
        "editsSha256",
        "generatedOutputSha256",
        "orderingSha256",
        "exitBehaviorSha256",
      ].toSorted(),
    );
});
