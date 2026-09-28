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
