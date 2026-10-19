import fs from "node:fs";
import path from "node:path";

import {
  array,
  deepEqual,
  enumValue,
  exactKeys,
  invalid,
  record,
  string,
  unique,
  type Fixture,
  type JsonObject,
} from "./fixture-compatibility-validation.ts";

export interface RegistryProject {
  readonly fixturePath: string;
  readonly note?: string;
  readonly repository?: string;
  readonly revision?: string;
  readonly vueGlobs?: readonly string[];
  readonly lspAuthoredOracle?: unknown;
  readonly typecheckPerformance?: { readonly enabled?: boolean };
  readonly [field: string]: unknown;
}

export interface DialectContext {
  readonly registry: { readonly projects: readonly RegistryProject[] };
  readonly rootDir: string;
}

// These are presence claims about a pinned project, not classifications of
// every source file in it. Per-case manifests must carry their own dialects.
export const dialectLabels = [
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

export function validateDialectCoverage(
  coverage: unknown,
  fixturePath: string,
  context: DialectContext,
): void {
  record(coverage, `${fixturePath}.dialectCoverage`);
  exactKeys(coverage, ["state", "evidence"]);
  enumValue(coverage.state, ["unknown", "partial"], "dialect coverage state");
  array(coverage.evidence, `${fixturePath}.dialectCoverage.evidence`);
  if ((coverage.state === "unknown") !== (coverage.evidence.length === 0)) {
    invalid(
      `${fixturePath} unknown dialect coverage must have no evidence; partial must have evidence`,
    );
  }
  const identities: string[] = [];
  for (const item of coverage.evidence) {
    record(item, `${fixturePath}.dialectCoverage.evidence`);
    if (item.kind === "registry-note") {
      exactKeys(item, ["kind", "dialects", "selector"]);
      const project = context.registry.projects.find((row) => row.fixturePath === fixturePath);
      if (project?.note !== item.selector) {
        invalid(`${fixturePath} dialect registry note is absent or stale`);
      }
    } else if (item.kind === "test-oracle") {
      exactKeys(item, ["kind", "dialects", "file", "selector"]);
      validateTestOracle(item, context.rootDir);
    } else if (item.kind === "pinned-package") {
      exactKeys(item, [
        "kind",
        "dialects",
        "repository",
        "revision",
        "file",
        "blobSha",
        "field",
        "value",
      ]);
      validatePinnedPackage(item, fixturePath, context);
    } else {
      invalid(`${fixturePath} unknown dialect evidence kind ${String(item.kind)}`);
    }
    array(item.dialects, `${fixturePath}.dialects`);
    if (item.dialects.length === 0) invalid(`${fixturePath} dialect evidence must name a dialect`);
    unique(item.dialects, `${fixturePath}.dialects`);
    for (const dialect of item.dialects) {
      enumValue(dialect, dialectLabels, "dialect label");
      identities.push(dialect);
    }
  }
  unique(identities, `${fixturePath} dialect claims`);
}

function validatePinnedPackage(
  item: JsonObject,
  fixturePath: string,
  context: DialectContext,
): void {
  const project = context.registry.projects.find((row) => row.fixturePath === fixturePath);
  if (project == null || !project.vueGlobs?.length) {
    invalid(`${fixturePath} pinned package claim needs a registered Vue source corpus`);
  }
  const { value, dialects } = item;
  string(value, `${fixturePath} pinned Vue package value`);
  array(dialects, `${fixturePath}.dialects`);
  if (item.repository !== project.repository || item.revision !== project.revision) {
    invalid(`${fixturePath} pinned package receipt drifted from fixture registry`);
  }
  if (item.file !== "package.json" || !/^[a-f0-9]{40}$/.test(String(item.blobSha))) {
    invalid(`${fixturePath} pinned package receipt must name a package.json Git blob`);
  }
  enumValue(
    item.field,
    ["dependencies.vue", "devDependencies.vue", "peerDependencies.vue"],
    "Vue package field",
  );
  if (!/^(?:\^|~)?2(?:\.(?:[0-9]+|x)(?:\.[0-9]+)?)?(?:$|\s)/.test(value)) {
    invalid(`${fixturePath} pinned Vue package value does not select Vue 2`);
  }
  if (dialects.includes("vue2.7-sfc") && !/(?:\^|~)?2\.7\./.test(value)) {
    invalid(`${fixturePath} Vue 2.7 claim needs a Vue 2.7 package value`);
  }
  if (dialects.some((dialect) => dialect !== "vue2-sfc" && dialect !== "vue2.7-sfc")) {
    invalid(`${fixturePath} package version cannot prove a template syntax dialect`);
  }
}

export function dialectCoverageSummary(fixtures: readonly Fixture[]) {
  return {
    unknownFixtureCount: fixtures.filter((fixture) => fixture.dialectCoverage.state === "unknown")
      .length,
    partialFixtureCount: fixtures.filter((fixture) => fixture.dialectCoverage.state === "partial")
      .length,
    presentInFixtures: Object.fromEntries(
      dialectLabels.map((dialect) => [
        dialect,
        fixtures
          .filter((fixture) =>
            fixture.dialectCoverage.evidence.some((item) => item.dialects.includes(dialect)),
          )
          .map((fixture) => fixture.fixturePath),
      ]),
    ),
  };
}

function validateTestOracle(item: JsonObject, rootDir: string): void {
  const { file, selector } = item;
  string(file, "dialect evidence file");
  string(selector, "dialect evidence selector");
  if (path.isAbsolute(file) || file.split(/[\\/]/).includes("..")) {
    invalid(`dialect evidence path must stay repository-relative: ${file}`);
  }
  const absolute = path.join(rootDir, file);
  if (!fs.statSync(absolute, { throwIfNoEntry: false })?.isFile()) {
    invalid(`dialect evidence file does not exist: ${file}`);
  }
  const occurrences = fs.readFileSync(absolute, "utf8").split(selector).length - 1;
  deepEqual(occurrences, 1, `dialect evidence selector is stale in ${file}`);
}
