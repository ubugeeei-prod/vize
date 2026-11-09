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

// Dependencies whose presence proves a dialect in a pinned project. The `vue`
// dependency proves the major its range selects.
const dialectPackages: Readonly<Record<string, string>> = {
  "petite-vue": "petite-vue",
  pug: "pug-template",
  "pug-plain-loader": "pug-template",
  "@vue/language-plugin-pug": "pug-template",
  "vue-pug-plugin": "pug-template",
  "@vitejs/plugin-vue-jsx": "jsx-babel",
  "@vue/babel-plugin-jsx": "jsx-babel",
  "@vue/babel-preset-jsx": "jsx-babel",
  "babel-plugin-transform-vue-jsx": "jsx-babel",
  "vue-jsx-vapor": "jsx-vapor",
};

export function packageDialect(name: string, value: string): string | undefined {
  if (name !== "vue") return dialectPackages[name];
  if (/^(?:\^|~)?2\.7\./.test(value)) return "vue2.7-sfc";
  if (/^(?:\^|~)?2(?:\.(?:[0-9]+|x)(?:\.[0-9]+)?)?(?:$|\s)/.test(value)) return "vue2-sfc";
  if (/^(?:\^|~|>=\s*)?3(?:\.(?:[0-9]+|x)(?:\.[0-9]+)?(?:-[0-9a-z.]+)?)?(?:$|\s)/.test(value)) {
    return "vue3-sfc";
  }
  return undefined;
}

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
        ...(item.catalog === undefined ? [] : ["catalog"]),
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
  const file = String(item.file);
  if (
    !/^(?:[^/]+\/)*package\.json$/.test(file) ||
    file.split("/").some((part) => part === ".." || part === "node_modules") ||
    !/^[a-f0-9]{40}$/.test(String(item.blobSha))
  ) {
    invalid(`${fixturePath} pinned package receipt must name a package.json Git blob`);
  }
  // A pnpm catalog reference names the workspace file and range it resolves to.
  let range = value;
  if (value.startsWith("catalog:")) {
    const catalog = item.catalog;
    record(catalog, `${fixturePath} catalog receipt`);
    exactKeys(catalog, ["file", "blobSha", "value"]);
    if (catalog.file !== "pnpm-workspace.yaml" || !/^[a-f0-9]{40}$/.test(String(catalog.blobSha))) {
      invalid(`${fixturePath} catalog receipt must name the pnpm-workspace.yaml Git blob`);
    }
    string(catalog.value, `${fixturePath} catalog range`);
    range = catalog.value;
  } else if (item.catalog !== undefined) {
    invalid(`${fixturePath} only a catalog reference carries a catalog receipt`);
  }
  const field = /^(?:dependencies|devDependencies|peerDependencies)\.(.+)$/.exec(
    String(item.field),
  );
  const dialect = field ? packageDialect(field[1]!, range) : undefined;
  if (dialect === undefined) invalid(`${fixturePath} ${String(item.field)} proves no dialect`);
  deepEqual(dialects, [dialect], `${fixturePath} ${String(item.field)} proves exactly ${dialect}`);
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
