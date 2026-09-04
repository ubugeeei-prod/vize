import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { fullAppE2eRows } from "../github/app-e2e-plan.mjs";
import {
  array,
  compareCodepoints,
  countMembership,
  deepEqual,
  evidenceIdentity,
  enumValue,
  equal,
  exactKeys,
  invalid,
  record,
  string,
  unique,
  validateCapabilityRuntimeClaims,
} from "./fixture-compatibility-validation.mjs";

const moduleRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
export const compatibilityLedgerPath = path.join(
  moduleRoot,
  "tests/_fixtures/fixture-compatibility-ledger.json",
);

export const capabilityValues = {
  "vue-generation": ["0.x", "1.x", "2.x", "2.7", "3.x"],
  "api-style": ["options-api", "class-api", "composition-api", "script-setup"],
  "nuxt-macro": ["define-page-meta", "use-head", "use-seo-meta"],
};
export const oracleKinds = [
  "compiler",
  "formatter-idempotency",
  "linter",
  "typechecker",
  "production-build",
  "authored-lsp",
  "vue-tsc-parity",
  "ssr",
  "hydration",
  "preview",
  "vrt",
  "real-vite-hmr",
];

const capabilityLevels = ["present", "exercised", "runtime"];
const memberships = ["ecosystem", "app"];
const unresolvedStates = ["unknown", "unverified", "excluded"];
const unresolvedDimensions = [
  "vue-generation",
  "vue-generation-runtime",
  "corpus-capability-classification",
  "oracle-coverage",
];
const unresolvedValues = {
  "vue-generation": capabilityValues["vue-generation"],
  "vue-generation-runtime": capabilityValues["vue-generation"],
  "corpus-capability-classification": ["remaining-gitlinks"],
  "oracle-coverage": ["preview"],
};
const runtimeOracleKinds = new Set(["ssr", "hydration", "preview", "vrt", "real-vite-hmr"]);

export function readCompatibilityLedger(file = compatibilityLedgerPath) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

export function createCompatibilityContext(rootDir = moduleRoot) {
  const registry = JSON.parse(
    fs.readFileSync(path.join(rootDir, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  );
  const gitmodulePaths = execFileSync(
    "git",
    ["config", "-f", ".gitmodules", "--get-regexp", "^submodule\\..*\\.path$"],
    { cwd: rootDir, encoding: "utf8" },
  )
    .trim()
    .split("\n")
    .map((line) => line.trim().split(/\s+/).at(-1))
    .sort(compareCodepoints);
  const gitlinkPaths = execFileSync("git", ["ls-files", "-s", "tests/_fixtures/_git"], {
    cwd: rootDir,
    encoding: "utf8",
  })
    .trim()
    .split("\n")
    .filter((line) => line.startsWith("160000 "))
    .map((line) => line.split("\t").at(-1))
    .sort(compareCodepoints);
  deepEqual(gitmodulePaths, gitlinkPaths, ".gitmodules and gitlink inventory drifted");
  return { appRows: fullAppE2eRows, gitlinkPaths, registry, rootDir };
}

export function validateCompatibilityLedger(ledger, context = createCompatibilityContext()) {
  record(ledger, "fixture compatibility ledger");
  exactKeys(ledger, [
    "schema",
    "version",
    "trackingIssues",
    "fixtures",
    "capabilities",
    "oracles",
    "unresolved",
  ]);
  equal(ledger.schema, "vize.fixtureCompatibilityLedger", "unsupported ledger schema");
  equal(ledger.version, 1, "unsupported ledger version");
  validateTrackingIssues(ledger.trackingIssues);

  const fixtureMap = validateFixtures(ledger.fixtures, context);
  validateCapabilities(ledger.capabilities, fixtureMap, context.rootDir);
  const { byKind: oracleFixtures, runtimeEvidence } = validateOracles(
    ledger.oracles,
    fixtureMap,
    context.rootDir,
  );
  validateCapabilityRuntimeClaims(ledger.capabilities, runtimeEvidence);
  validateUnresolved(ledger.unresolved);
  validateRatchets(oracleFixtures, fixtureMap, context);
  return { fixtureMap, oracleFixtures };
}

function validateTrackingIssues(value) {
  record(value, "trackingIssues");
  exactKeys(value, ["authoredLspExpansion", "vueTscParityExpansion", "dropInCompatibility"]);
  equal(value.authoredLspExpansion, 3952, "authored LSP ownership drifted");
  equal(value.vueTscParityExpansion, 3954, "vue-tsc parity ownership drifted");
  equal(value.dropInCompatibility, 3227, "drop-in compatibility ownership drifted");
}

function validateFixtures(fixtures, context) {
  array(fixtures, "fixtures");
  const expectedGitlinks = context.gitlinkPaths;
  const actualPaths = fixtures.map((fixture, index) => {
    record(fixture, `fixtures[${index}]`);
    exactKeys(fixture, ["fixturePath", "memberships"]);
    string(fixture.fixturePath, `fixtures[${index}].fixturePath`);
    array(fixture.memberships, `${fixture.fixturePath}.memberships`);
    unique(fixture.memberships, `${fixture.fixturePath}.memberships`);
    for (const membership of fixture.memberships) enumValue(membership, memberships, "membership");
    return fixture.fixturePath;
  });
  unique(actualPaths, "fixture paths");
  deepEqual(
    actualPaths,
    [...actualPaths].sort(compareCodepoints),
    "fixture paths must be codepoint sorted",
  );
  deepEqual(actualPaths, expectedGitlinks, "ledger fixtures must exactly partition .gitmodules");

  const ecosystem = new Set(context.registry.projects.map((project) => project.fixturePath));
  const app = new Set(context.appRows.flatMap((row) => row.fixtures));
  const fixtureMap = new Map(fixtures.map((fixture) => [fixture.fixturePath, fixture]));
  for (const fixture of fixtures) {
    const expected = [
      ...(ecosystem.has(fixture.fixturePath) ? ["ecosystem"] : []),
      ...(app.has(fixture.fixturePath) ? ["app"] : []),
    ];
    deepEqual(fixture.memberships, expected, `${fixture.fixturePath} membership drifted`);
  }
  return fixtureMap;
}

function validateCapabilities(capabilities, fixtureMap, rootDir) {
  array(capabilities, "capabilities");
  const identities = [];
  for (const [index, capability] of capabilities.entries()) {
    record(capability, `capabilities[${index}]`);
    exactKeys(capability, ["fixturePath", "dimension", "value", "levels", "evidence"]);
    if (!fixtureMap.has(capability.fixturePath))
      invalid(`unknown fixture ${capability.fixturePath}`);
    const values = capabilityValues[capability.dimension];
    if (values == null) invalid(`unknown capability dimension ${capability.dimension}`);
    enumValue(capability.value, values, `${capability.dimension} value`);
    array(capability.levels, "capability levels");
    if (capability.levels.length === 0) invalid("capability levels must not be empty");
    unique(capability.levels, "capability levels");
    for (const level of capability.levels) enumValue(level, capabilityLevels, "capability level");
    if (
      capability.levels.includes("runtime") &&
      (!capability.levels.includes("present") || !capability.levels.includes("exercised"))
    ) {
      invalid("runtime capability must also be present and exercised");
    }
    if (
      capability.levels.includes("runtime") &&
      !fixtureMap.get(capability.fixturePath).memberships.includes("app")
    ) {
      invalid(`runtime capability claim is not an App fixture: ${capability.fixturePath}`);
    }
    validateEvidence(capability.evidence, rootDir);
    identities.push(`${capability.fixturePath}\0${capability.dimension}\0${capability.value}`);
  }
  unique(identities, "capability claims");
}

function validateOracles(oracles, fixtureMap, rootDir) {
  array(oracles, "oracles");
  const identities = [];
  const byKind = new Map(oracleKinds.map((kind) => [kind, new Set()]));
  const runtimeEvidence = new Set();
  for (const [index, oracle] of oracles.entries()) {
    record(oracle, `oracles[${index}]`);
    exactKeys(oracle, ["kind", "selection", "evidence"]);
    enumValue(oracle.kind, oracleKinds, "oracle kind");
    const selected = expandSelection(oracle.selection, fixtureMap);
    if (selected.length === 0) invalid(`${oracle.kind} oracle selected no fixtures`);
    validateEvidence(oracle.evidence, rootDir);
    for (const fixturePath of selected) {
      identities.push(`${oracle.kind}\0${fixturePath}`);
      byKind.get(oracle.kind).add(fixturePath);
      if (runtimeOracleKinds.has(oracle.kind)) {
        runtimeEvidence.add(evidenceIdentity(fixturePath, oracle.evidence));
      }
      if (
        ["production-build", "ssr", "hydration", "preview", "vrt", "real-vite-hmr"].includes(
          oracle.kind,
        ) &&
        !fixtureMap.get(fixturePath).memberships.includes("app")
      ) {
        invalid(`${oracle.kind} App oracle is not an App fixture: ${fixturePath}`);
      }
    }
  }
  unique(identities, "oracle fixture claims");
  return { byKind, runtimeEvidence };
}

export function expandSelection(selection, fixtureMap) {
  record(selection, "oracle selection");
  if (selection.type === "membership") {
    exactKeys(selection, ["type", "membership"]);
    enumValue(selection.membership, memberships, "selection membership");
    return [...fixtureMap.values()]
      .filter((fixture) => fixture.memberships.includes(selection.membership))
      .map((fixture) => fixture.fixturePath);
  }
  if (selection.type === "fixtures") {
    exactKeys(selection, ["type", "fixturePaths"]);
    array(selection.fixturePaths, "selection.fixturePaths");
    unique(selection.fixturePaths, "selection.fixturePaths");
    for (const fixturePath of selection.fixturePaths) {
      if (!fixtureMap.has(fixturePath)) invalid(`oracle selected unknown fixture ${fixturePath}`);
    }
    return selection.fixturePaths;
  }
  invalid(`unknown oracle selection type ${selection.type}`);
}

function validateEvidence(evidence, rootDir) {
  record(evidence, "evidence");
  exactKeys(evidence, ["file", "selector"]);
  string(evidence.file, "evidence.file");
  string(evidence.selector, "evidence.selector");
  if (path.isAbsolute(evidence.file) || evidence.file.split(/[\\/]/).includes("..")) {
    invalid(`evidence path must stay repository-relative: ${evidence.file}`);
  }
  const absolute = path.join(rootDir, evidence.file);
  if (!fs.statSync(absolute, { throwIfNoEntry: false })?.isFile()) {
    invalid(`evidence file does not exist: ${evidence.file}`);
  }
  const occurrences = fs.readFileSync(absolute, "utf8").split(evidence.selector).length - 1;
  if (occurrences !== 1) {
    invalid(`evidence selector is stale in ${evidence.file}: ${evidence.selector}`);
  }
}

function validateUnresolved(unresolved) {
  array(unresolved, "unresolved");
  const identities = [];
  const rows = new Map();
  for (const [index, item] of unresolved.entries()) {
    record(item, `unresolved[${index}]`);
    exactKeys(item, ["dimension", "value", "state", "reason", "trackingIssue"]);
    enumValue(item.dimension, unresolvedDimensions, "unresolved dimension");
    enumValue(item.value, unresolvedValues[item.dimension], `${item.dimension} unresolved value`);
    enumValue(item.state, unresolvedStates, "unresolved state");
    string(item.reason, "unresolved reason");
    if (!Number.isInteger(item.trackingIssue) || item.trackingIssue <= 0) {
      invalid("unresolved trackingIssue must be a positive integer");
    }
    const identity = `${item.dimension}\0${item.value}`;
    identities.push(identity);
    rows.set(identity, item);
  }
  unique(identities, "unresolved dimensions");
  const required = new Map([
    [["vue-generation", "0.x"].join("\u0000"), "unverified"],
    [["vue-generation", "1.x"].join("\u0000"), "unverified"],
    [["vue-generation-runtime", "2.7"].join("\u0000"), "unverified"],
    [["corpus-capability-classification", "remaining-gitlinks"].join("\u0000"), "unknown"],
    [["oracle-coverage", "preview"].join("\u0000"), "unverified"],
  ]);
  for (const [identity, state] of required) {
    const row = rows.get(identity);
    if (row == null) invalid(`missing explicit unresolved dimension ${identity}`);
    equal(row.state, state, `unresolved state drifted for ${identity}`);
    equal(row.trackingIssue, 3227, `unresolved owner drifted for ${identity}`);
  }
}

function validateRatchets(oracles, fixtureMap, context) {
  equal(fixtureMap.size, 146, "gitlink count drifted");
  equal(countMembership(fixtureMap, "ecosystem"), 142, "ecosystem fixture count drifted");
  equal(countMembership(fixtureMap, "app"), 16, "App fixture count drifted");
  const appOnly = [...fixtureMap.values()].filter(
    (fixture) => fixture.memberships.length === 1 && fixture.memberships[0] === "app",
  );
  deepEqual(
    appOnly.map((fixture) => fixture.fixturePath),
    [
      "tests/_fixtures/_git/frontend-phpcon-do-website",
      "tests/_fixtures/_git/npmx.dev",
      "tests/_fixtures/_git/nuxt-ui",
      "tests/_fixtures/_git/vuefes-2025",
    ],
    "App-only fixture membership drifted",
  );
  for (const kind of ["compiler", "formatter-idempotency", "linter", "typechecker"]) {
    equal(oracles.get(kind).size, 142, `${kind} oracle count drifted`);
  }
  equal(oracles.get("authored-lsp").size, 21, "authored LSP oracle count drifted");
  equal(oracles.get("vue-tsc-parity").size, 11, "vue-tsc parity count drifted");
  deepEqual(
    [...oracles.get("authored-lsp")].sort(compareCodepoints),
    context.registry.projects
      .filter((project) => project.lspAuthoredOracle != null)
      .map((project) => project.fixturePath)
      .sort(compareCodepoints),
    "authored LSP oracle membership drifted",
  );
  deepEqual(
    [...oracles.get("vue-tsc-parity")].sort(compareCodepoints),
    [
      ...new Set(
        context.registry.projects
          .filter((project) => project.typecheckPerformance?.enabled === true)
          .map((project) => project.fixturePath),
      ),
    ].sort(compareCodepoints),
    "vue-tsc parity membership drifted",
  );
}
