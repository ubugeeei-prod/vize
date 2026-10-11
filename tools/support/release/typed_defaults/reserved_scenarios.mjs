import assert from "node:assert/strict";
import path from "node:path";
import { sha256 } from "./custody.mjs";

export const reservedDirectories = {
  bare: "tests/_fixtures/differential/lint/bare-reserved-8090",
  prefix: "tests/_fixtures/differential/lint/reserved-prefix-8090",
  alias: "tests/_fixtures/differential/lint/reserved-suppression-8090",
};
export const reservedRecipeFiles = [
  `${reservedDirectories.bare}/cases.json`,
  `${reservedDirectories.bare}/original-wave-ui-accordion.json`,
  `${reservedDirectories.prefix}/bare-parent-route-successors.json`,
  `${reservedDirectories.prefix}/cases.json`,
  `${reservedDirectories.prefix}/default-route-companions.json`,
  `${reservedDirectories.prefix}/originals.json`,
  `${reservedDirectories.alias}/original-prefix-successors.json`,
  `${reservedDirectories.alias}/cases.json`,
  `${reservedDirectories.alias}/default-route-successors.json`,
  `${reservedDirectories.alias}/original-vue-select-successor.json`,
  "tests/_fixtures/n8n-cli-adoption.json",
];

/** Compile the existing literal whole-route successors; never derive findings from a process. */
export function reservedScenarios(read) {
  const json = (name) => JSON.parse(read(name).toString("utf8"));
  const projection = json("tests/_fixtures/n8n-cli-adoption.json");
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  const vectors = [],
    sourcePaths = new Set();
  function carrier(directory, entry) {
    const name = path.posix.normalize(
      `${directory}/${entry.sourceFile ?? entry.filename + ".fixture"}`,
    );
    assert(name.startsWith("tests/_fixtures/differential/lint/"));
    assert.match(entry.filename, /^[A-Za-z0-9_.-]+\.vue$/);
    const bytes = read(name);
    assert.equal(sha256(bytes), entry.sourceSha256, entry.id);
    if (entry.source !== undefined) assert.equal(bytes.toString("utf8"), entry.source);
    sourcePaths.add(name);
    return { ...entry, source: bytes.toString("utf8") };
  }
  function seven(entry, rule, suite, routes) {
    assert.equal(projection.linter.rules[rule], undefined);
    vectors.push(
      ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({
        entry,
        preset,
        config: null,
        suite,
        findings: preset === "essential" ? routes.essential : routes.default,
      })),
      { entry, preset: "incremental", config: null, findings: [], suite },
      {
        entry,
        preset: null,
        config: JSON.stringify({ linter: { rules: { [rule]: "off" } } }) + "\n",
        findings: routes.off,
        suite,
      },
      {
        entry,
        preset: null,
        config: JSON.stringify({ linter: projection.linter }) + "\n",
        findings: routes.projection,
        suite,
      },
    );
  }
  function original(entry, rule, findings, suite) {
    vectors.push({
      entry,
      preset: null,
      findings,
      suite,
      config:
        JSON.stringify({ linter: { preset: "incremental", rules: { [rule]: "error" } } }) + "\n",
    });
  }
  const bare = json(`${reservedDirectories.bare}/cases.json`);
  const bareRoutes = json(`${reservedDirectories.prefix}/bare-parent-route-successors.json`);
  assert.equal(bare.cases.length, 16);
  assert.equal(bareRoutes.cases.length, 16);
  bare.cases.forEach((entry, index) => {
    const route = bareRoutes.cases[index];
    assert.equal(route.id, entry.id);
    assert.equal(route.sourceSha256, entry.sourceSha256);
    seven(carrier(reservedDirectories.bare, entry), bare.rule, "reserved-bare", {
      default: route.combinedCliDiagnostics,
      essential: route.combinedCliDiagnostics,
      off: route.diagnostics,
      projection: [],
    });
  });
  const wave = json(`${reservedDirectories.bare}/original-wave-ui-accordion.json`);
  original(carrier(reservedDirectories.bare, wave), bare.rule, wave.diagnostics, "reserved-bare");

  const prefix = json(`${reservedDirectories.prefix}/cases.json`);
  const companions = json(`${reservedDirectories.prefix}/default-route-companions.json`);
  assert.equal(prefix.cases.length, 20);
  assert.equal(companions.cases.length, 20);
  prefix.cases.forEach((entry, index) => {
    const route = companions.cases[index];
    assert.equal(route.id, entry.id);
    assert.equal(route.sourceSha256, entry.sourceSha256);
    seven(carrier(reservedDirectories.prefix, entry), prefix.rule, "reserved-prefix", {
      default: route.combinedCliDiagnostics,
      essential: route.combinedCliDiagnostics,
      off: route.diagnostics,
      projection: [],
    });
  });
  const originals = json(`${reservedDirectories.prefix}/originals.json`);
  const prefixSuccessors = json(`${reservedDirectories.alias}/original-prefix-successors.json`);
  assert.equal(originals.cases.length, 21);
  assert.equal(prefixSuccessors.cases.length, 21);
  originals.cases.forEach((entry, index) => {
    const successor = prefixSuccessors.cases[index];
    assert.equal(successor.id, entry.id);
    assert.equal(successor.sourceSha256, entry.sourceSha256);
    original(
      carrier(reservedDirectories.prefix, entry),
      prefix.rule,
      successor.diagnostics,
      "reserved-prefix",
    );
  });

  const alias = json(`${reservedDirectories.alias}/cases.json`);
  const aliasRoutes = json(`${reservedDirectories.alias}/default-route-successors.json`);
  assert.equal(alias.cases.length, 24);
  assert.equal(aliasRoutes.cases.length, 24);
  alias.cases.forEach((entry, index) => {
    const route = aliasRoutes.cases[index];
    assert.equal(route.id, entry.id);
    assert.equal(route.sourceSha256, entry.sourceSha256);
    seven(carrier(reservedDirectories.alias, entry), alias.rule, "reserved-alias", {
      default: route.defaultDiagnostics,
      essential: route.essentialDiagnostics,
      off: route.offDiagnostics,
      projection: route.projectionDiagnostics,
    });
    assert.deepEqual(route.incrementalDiagnostics, []);
  });
  const select = json(`${reservedDirectories.alias}/original-vue-select-successor.json`);
  original(
    carrier(reservedDirectories.alias, select),
    alias.rule,
    select.diagnostics,
    "reserved-alias",
  );
  assert.equal(vectors.length, 443);
  for (const [suite, count] of [
    ["reserved-bare", 113],
    ["reserved-prefix", 161],
    ["reserved-alias", 169],
  ])
    assert.equal(vectors.filter((v) => v.suite === suite).length, count);
  return { vectors, sourcePaths: [...sourcePaths] };
}

export function reservedNormalized(vectors, wholeJson) {
  return vectors.map(({ entry, preset, config, findings, suite }) => ({
    suite,
    id: entry.id,
    filename: entry.filename,
    source: entry.source,
    preset,
    config,
    expected: wholeJson(entry, findings),
    expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
  }));
}
