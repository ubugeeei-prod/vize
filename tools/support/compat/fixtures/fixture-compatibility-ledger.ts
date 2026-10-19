import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { fullAppE2eRows, type AppE2eRow } from "../github/app-e2e-plan.ts";
import {
  validateDialectCoverage,
  type DialectContext,
  type RegistryProject,
} from "./fixture-dialect-coverage.ts";
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
  type Capability,
  type Evidence,
  type Fixture,
} from "./fixture-compatibility-validation.ts";

export interface CompatibilityContext extends DialectContext {
  readonly appRows: readonly AppE2eRow[];
  readonly gitlinkPaths: readonly string[];
}

type FixtureMap = ReadonlyMap<string, Fixture>;

export interface UnresolvedItem {
  readonly dimension: string;
  readonly value: string;
  readonly state: string;
  readonly reason: string;
  readonly trackingIssue: number;
}

/** The checked-in JSON document as read; nothing is validated yet. */
export interface LedgerDocument {
  fixtures: {
    fixturePath: string;
    memberships: string[];
    dialectCoverage: Record<string, unknown>;
  }[];
  capabilities: {
    fixturePath: string;
    dimension: string;
    value: string;
    levels: string[];
    evidence: Record<string, unknown>;
  }[];
  oracles: {
    kind: string;
    selection: Record<string, unknown>;
    evidence: Record<string, unknown>;
  }[];
  unresolved: Record<string, unknown>[];
  [field: string]: unknown;
}

/** The ledger after `validateCompatibilityLedger` checked every closed shape. */
export interface CompatibilityLedger {
  readonly fixtures: readonly Fixture[];
  readonly capabilities: readonly Capability[];
  readonly oracles: readonly {
    readonly kind: string;
    readonly selection: unknown;
    readonly evidence: Evidence;
  }[];
  readonly unresolved: readonly UnresolvedItem[];
}

const moduleRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
export const compatibilityLedgerPath = path.join(
  moduleRoot,
  "tests/_fixtures/fixture-compatibility-ledger.json",
);

export const capabilityValues: Readonly<Record<string, readonly string[]>> = {
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
const unresolvedValues: Readonly<Record<string, readonly string[]>> = {
  "vue-generation": capabilityValues["vue-generation"],
  "vue-generation-runtime": capabilityValues["vue-generation"],
  "corpus-capability-classification": ["remaining-gitlinks"],
  "oracle-coverage": ["preview"],
};

function lastField(line: string, separator: RegExp | string): string {
  return line.trim().split(separator).at(-1) ?? "";
}
const runtimeOracleKinds = new Set(["ssr", "hydration", "preview", "vrt", "real-vite-hmr"]);

export function readCompatibilityLedger(file = compatibilityLedgerPath): LedgerDocument {
  // Callers pass the document to validateCompatibilityLedger, which checks it.
  return JSON.parse(fs.readFileSync(file, "utf8")) as LedgerDocument;
}

export function createCompatibilityContext(rootDir = moduleRoot): CompatibilityContext {
  // The checked-in registry is validated by its own fixture-registry tests.
  const registry = JSON.parse(
    fs.readFileSync(path.join(rootDir, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  ) as { projects: readonly RegistryProject[] };
  const gitmodulePaths = execFileSync(
    "git",
    ["config", "-f", ".gitmodules", "--get-regexp", "^submodule\\..*\\.path$"],
    { cwd: rootDir, encoding: "utf8" },
  )
    .trim()
    .split("\n")
    .map((line) => lastField(line, /\s+/))
    .sort(compareCodepoints);
  const gitlinkPaths = execFileSync("git", ["ls-files", "-s", "tests/_fixtures/_git"], {
    cwd: rootDir,
    encoding: "utf8",
  })
    .trim()
    .split("\n")
    .filter((line) => line.startsWith("160000 "))
    .map((line) => lastField(line, "\t"))
    .sort(compareCodepoints);
  deepEqual(gitmodulePaths, gitlinkPaths, ".gitmodules and gitlink inventory drifted");
  return { appRows: fullAppE2eRows, gitlinkPaths, registry, rootDir };
}

export function validateCompatibilityLedger(
  ledger: unknown,
  context: CompatibilityContext = createCompatibilityContext(),
): {
  ledger: CompatibilityLedger;
  fixtureMap: Map<string, Fixture>;
  oracleFixtures: Map<string, Set<string>>;
} {
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
  const capabilities = validateCapabilities(ledger.capabilities, fixtureMap, context.rootDir);
  const { byKind: oracleFixtures, runtimeEvidence } = validateOracles(
    ledger.oracles,
    fixtureMap,
    context.rootDir,
  );
  validateCapabilityRuntimeClaims(capabilities, runtimeEvidence);
  validateUnresolved(ledger.unresolved);
  validateRatchets(oracleFixtures, fixtureMap, context);
  return { ledger: ledger as unknown as CompatibilityLedger, fixtureMap, oracleFixtures };
}

function validateTrackingIssues(value: unknown): void {
  record(value, "trackingIssues");
  exactKeys(value, ["authoredLspExpansion", "vueTscParityExpansion", "dropInCompatibility"]);
  equal(value.authoredLspExpansion, 3952, "authored LSP ownership drifted");
  equal(value.vueTscParityExpansion, 3954, "vue-tsc parity ownership drifted");
  equal(value.dropInCompatibility, 3227, "drop-in compatibility ownership drifted");
}

function validateFixtures(fixtures: unknown, context: CompatibilityContext): Map<string, Fixture> {
  array(fixtures, "fixtures");
  const expectedGitlinks = context.gitlinkPaths;
  const validated: Fixture[] = [];
  const actualPaths = fixtures.map((fixture, index) => {
    record(fixture, `fixtures[${index}]`);
    exactKeys(fixture, ["fixturePath", "memberships", "dialectCoverage"]);
    string(fixture.fixturePath, `fixtures[${index}].fixturePath`);
    array(fixture.memberships, `${fixture.fixturePath}.memberships`);
    unique(fixture.memberships, `${fixture.fixturePath}.memberships`);
    for (const membership of fixture.memberships) enumValue(membership, memberships, "membership");
    validateDialectCoverage(fixture.dialectCoverage, fixture.fixturePath, context);
    // Every field of the closed shape was checked above.
    validated.push(fixture as unknown as Fixture);
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
  const fixtureMap = new Map(validated.map((fixture) => [fixture.fixturePath, fixture]));
  for (const fixture of validated) {
    const expected = [
      ...(ecosystem.has(fixture.fixturePath) ? ["ecosystem"] : []),
      ...(app.has(fixture.fixturePath) ? ["app"] : []),
    ];
    deepEqual(fixture.memberships, expected, `${fixture.fixturePath} membership drifted`);
  }
  return fixtureMap;
}

function validateCapabilities(
  capabilities: unknown,
  fixtureMap: FixtureMap,
  rootDir: string,
): Capability[] {
  array(capabilities, "capabilities");
  const identities: string[] = [];
  const validated: Capability[] = [];
  for (const [index, capability] of capabilities.entries()) {
    record(capability, `capabilities[${index}]`);
    exactKeys(capability, ["fixturePath", "dimension", "value", "levels", "evidence"]);
    const { fixturePath, dimension } = capability;
    string(fixturePath, "capability fixturePath");
    string(dimension, "capability dimension");
    const fixture = fixtureMap.get(fixturePath);
    if (fixture === undefined) return invalid(`unknown fixture ${fixturePath}`);
    const values = capabilityValues[dimension];
    if (values == null) return invalid(`unknown capability dimension ${dimension}`);
    enumValue(capability.value, values, `${dimension} value`);
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
    if (capability.levels.includes("runtime") && !fixture.memberships.includes("app")) {
      invalid(`runtime capability claim is not an App fixture: ${fixturePath}`);
    }
    validateEvidence(capability.evidence, rootDir);
    identities.push(`${fixturePath}\0${dimension}\0${capability.value}`);
    // Every field of the closed shape was checked above.
    validated.push(capability as unknown as Capability);
  }
  unique(identities, "capability claims");
  return validated;
}

function validateOracles(
  oracles: unknown,
  fixtureMap: FixtureMap,
  rootDir: string,
): { byKind: Map<string, Set<string>>; runtimeEvidence: Set<string> } {
  array(oracles, "oracles");
  const identities: string[] = [];
  const byKind = new Map(oracleKinds.map((kind) => [kind, new Set<string>()]));
  const runtimeEvidence = new Set<string>();
  for (const [index, oracle] of oracles.entries()) {
    record(oracle, `oracles[${index}]`);
    exactKeys(oracle, ["kind", "selection", "evidence"]);
    enumValue(oracle.kind, oracleKinds, "oracle kind");
    const selected = expandSelection(oracle.selection, fixtureMap);
    if (selected.length === 0) invalid(`${oracle.kind} oracle selected no fixtures`);
    const evidence = validateEvidence(oracle.evidence, rootDir);
    for (const fixturePath of selected) {
      identities.push(`${oracle.kind}\0${fixturePath}`);
      byKind.get(oracle.kind)?.add(fixturePath);
      if (runtimeOracleKinds.has(oracle.kind)) {
        runtimeEvidence.add(evidenceIdentity(fixturePath, evidence));
      }
      if (
        ["production-build", "ssr", "hydration", "preview", "vrt", "real-vite-hmr"].includes(
          oracle.kind,
        ) &&
        fixtureMap.get(fixturePath)?.memberships.includes("app") !== true
      ) {
        invalid(`${oracle.kind} App oracle is not an App fixture: ${fixturePath}`);
      }
    }
  }
  unique(identities, "oracle fixture claims");
  return { byKind, runtimeEvidence };
}

export function expandSelection(selection: unknown, fixtureMap: FixtureMap): readonly string[] {
  record(selection, "oracle selection");
  if (selection.type === "membership") {
    exactKeys(selection, ["type", "membership"]);
    const { membership } = selection;
    enumValue(membership, memberships, "selection membership");
    return [...fixtureMap.values()]
      .filter((fixture) => fixture.memberships.includes(membership))
      .map((fixture) => fixture.fixturePath);
  }
  if (selection.type === "fixtures") {
    exactKeys(selection, ["type", "fixturePaths"]);
    const { fixturePaths } = selection;
    array(fixturePaths, "selection.fixturePaths");
    unique(fixturePaths, "selection.fixturePaths");
    const selected: string[] = [];
    for (const fixturePath of fixturePaths) {
      if (typeof fixturePath !== "string" || !fixtureMap.has(fixturePath))
        return invalid(`oracle selected unknown fixture ${String(fixturePath)}`);
      selected.push(fixturePath);
    }
    return selected;
  }
  return invalid(`unknown oracle selection type ${String(selection.type)}`);
}

function validateEvidence(evidence: unknown, rootDir: string): Evidence {
  record(evidence, "evidence");
  exactKeys(evidence, ["file", "selector"]);
  const { file, selector } = evidence;
  string(file, "evidence.file");
  string(selector, "evidence.selector");
  if (path.isAbsolute(file) || file.split(/[\\/]/).includes("..")) {
    invalid(`evidence path must stay repository-relative: ${file}`);
  }
  const absolute = path.join(rootDir, file);
  if (!fs.statSync(absolute, { throwIfNoEntry: false })?.isFile()) {
    invalid(`evidence file does not exist: ${file}`);
  }
  const occurrences = fs.readFileSync(absolute, "utf8").split(selector).length - 1;
  if (occurrences !== 1) {
    invalid(`evidence selector is stale in ${file}: ${selector}`);
  }
  return { file, selector };
}

function validateUnresolved(unresolved: unknown): void {
  array(unresolved, "unresolved");
  const identities: string[] = [];
  const rows = new Map<string, Readonly<Record<string, unknown>>>();
  for (const [index, item] of unresolved.entries()) {
    record(item, `unresolved[${index}]`);
    exactKeys(item, ["dimension", "value", "state", "reason", "trackingIssue"]);
    enumValue(item.dimension, unresolvedDimensions, "unresolved dimension");
    enumValue(
      item.value,
      unresolvedValues[item.dimension] ?? [],
      `${item.dimension} unresolved value`,
    );
    enumValue(item.state, unresolvedStates, "unresolved state");
    string(item.reason, "unresolved reason");
    if (
      typeof item.trackingIssue !== "number" ||
      !Number.isInteger(item.trackingIssue) ||
      item.trackingIssue <= 0
    ) {
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
    if (row == null) return invalid(`missing explicit unresolved dimension ${identity}`);
    equal(row.state, state, `unresolved state drifted for ${identity}`);
    equal(row.trackingIssue, 3227, `unresolved owner drifted for ${identity}`);
  }
}

function validateRatchets(
  oracles: ReadonlyMap<string, ReadonlySet<string>>,
  fixtureMap: FixtureMap,
  context: CompatibilityContext,
): void {
  equal(fixtureMap.size, 148, "gitlink count drifted");
  equal(countMembership(fixtureMap, "ecosystem"), 143, "ecosystem fixture count drifted");
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
    equal(oracles.get(kind)?.size, 143, `${kind} oracle count drifted`);
  }
  const authoredLsp = oracles.get("authored-lsp") ?? new Set<string>();
  const vueTscParity = oracles.get("vue-tsc-parity") ?? new Set<string>();
  equal(authoredLsp.size, 55, "authored LSP oracle count drifted");
  equal(vueTscParity.size, 11, "vue-tsc parity count drifted");
  deepEqual(
    [...authoredLsp].sort(compareCodepoints),
    [
      ...new Set(
        context.registry.projects
          .filter((project) => project.lspAuthoredOracle != null)
          .map((project) => project.fixturePath),
      ),
    ].sort(compareCodepoints),
    "authored LSP oracle membership drifted",
  );
  deepEqual(
    [...vueTscParity].sort(compareCodepoints),
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
