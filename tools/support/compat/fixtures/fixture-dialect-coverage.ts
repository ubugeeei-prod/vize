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
} from "./fixture-compatibility-validation.mjs";

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

export function validateDialectCoverage(coverage, fixturePath, context) {
  record(coverage, `${fixturePath}.dialectCoverage`);
  exactKeys(coverage, ["state", "evidence"]);
  enumValue(coverage.state, ["unknown", "partial"], "dialect coverage state");
  array(coverage.evidence, `${fixturePath}.dialectCoverage.evidence`);
  if ((coverage.state === "unknown") !== (coverage.evidence.length === 0)) {
    invalid(
      `${fixturePath} unknown dialect coverage must have no evidence; partial must have evidence`,
    );
  }
  const identities = [];
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
      invalid(`${fixturePath} unknown dialect evidence kind ${item.kind}`);
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

function validatePinnedPackage(item, fixturePath, context) {
  const project = context.registry.projects.find((row) => row.fixturePath === fixturePath);
  if (project == null || !project.vueGlobs?.length) {
    invalid(`${fixturePath} pinned package claim needs a registered Vue source corpus`);
  }
  if (item.repository !== project.repository || item.revision !== project.revision) {
    invalid(`${fixturePath} pinned package receipt drifted from fixture registry`);
  }
  if (item.file !== "package.json" || !/^[a-f0-9]{40}$/.test(item.blobSha)) {
    invalid(`${fixturePath} pinned package receipt must name a package.json Git blob`);
  }
  enumValue(
    item.field,
    ["dependencies.vue", "devDependencies.vue", "peerDependencies.vue"],
    "Vue package field",
  );
  if (!/^(?:\^|~)?2(?:\.(?:[0-9]+|x)(?:\.[0-9]+)?)?(?:$|\s)/.test(item.value)) {
    invalid(`${fixturePath} pinned Vue package value does not select Vue 2`);
  }
  if (item.dialects.includes("vue2.7-sfc") && !/(?:\^|~)?2\.7\./.test(item.value)) {
    invalid(`${fixturePath} Vue 2.7 claim needs a Vue 2.7 package value`);
  }
  if (item.dialects.some((dialect) => !["vue2-sfc", "vue2.7-sfc"].includes(dialect))) {
    invalid(`${fixturePath} package version cannot prove a template syntax dialect`);
  }
}

export function dialectCoverageSummary(fixtures) {
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

function validateTestOracle(item, rootDir) {
  string(item.file, "dialect evidence file");
  string(item.selector, "dialect evidence selector");
  if (path.isAbsolute(item.file) || item.file.split(/[\\/]/).includes("..")) {
    invalid(`dialect evidence path must stay repository-relative: ${item.file}`);
  }
  const absolute = path.join(rootDir, item.file);
  if (!fs.statSync(absolute, { throwIfNoEntry: false })?.isFile()) {
    invalid(`dialect evidence file does not exist: ${item.file}`);
  }
  const occurrences = fs.readFileSync(absolute, "utf8").split(item.selector).length - 1;
  deepEqual(occurrences, 1, `dialect evidence selector is stale in ${item.file}`);
}
