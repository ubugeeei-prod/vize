import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  githubHeadingAnchor,
  readWorkflowEntries,
  renderWorkflowCatalog,
  workflowGroupId,
} from "../../tools/commands/ci/workflow-catalog.ts";
import { parseTapCounts } from "../../tools/support/github/docs-preview-readiness.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

const embeddedProgram =
  /(?:node|python\d*)\s+(?:--input-type=module\s+)?-\s+<<|(?:node|python\d*)\s+--input-type=module\s+<</g;

const approvedRunnerLabels = new Set([
  "blacksmith-12vcpu-macos-15",
  "blacksmith-32vcpu-ubuntu-2204",
  "blacksmith-32vcpu-ubuntu-2404",
  "blacksmith-32vcpu-windows-2025",
  // Compatibility jobs need runner architectures or images unavailable from Blacksmith.
  "macos-15-intel",
  "ubuntu-22.04",
  "ubuntu-22.04-arm",
  "windows-11-arm",
  "windows-2025-vs2026",
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return value != null && typeof value === "object" && !Array.isArray(value);
}

function valuesForMatrixKey(value: unknown, key: string): string[] {
  if (Array.isArray(value)) return value.flatMap((entry) => valuesForMatrixKey(entry, key));
  if (!isRecord(value)) return [];
  return Object.entries(value).flatMap(([candidate, entry]) =>
    candidate === key && typeof entry === "string" ? [entry] : valuesForMatrixKey(entry, key),
  );
}

function workflowRunnerLabels(source: string): string[] {
  const workflow = parse(source) as unknown;
  assert.ok(isRecord(workflow) && isRecord(workflow.jobs), "workflow must define jobs");
  return Object.values(workflow.jobs).flatMap((value) => {
    assert.ok(isRecord(value), "workflow job must be a mapping");
    const declared = Array.isArray(value["runs-on"]) ? value["runs-on"] : [value["runs-on"]];
    const strategy = isRecord(value.strategy) ? value.strategy : {};
    return declared.flatMap((runner) => {
      if (typeof runner !== "string") return [];
      const matrixKeys = [...runner.matchAll(/matrix\.(?:[A-Za-z0-9_-]+\.)*([A-Za-z0-9_-]+)/g)].map(
        (match) => match[1]!,
      );
      if (matrixKeys.length === 0) return [runner];
      const resolved = matrixKeys.flatMap((key) => valuesForMatrixKey(strategy.matrix, key));
      if (resolved.length === 0 && typeof strategy.matrix === "string") {
        // Generated matrices are validated by their typed planner contract.
        assert.match(strategy.matrix, /^\$\{\{\s*fromJSON\(.+\)\s*\}\}$/u);
        return [];
      }
      assert.ok(resolved.length > 0, `cannot resolve runner matrix expression: ${runner}`);
      return resolved;
    });
  });
}

test("the flat GitHub workflow directory has a generated navigation catalog", () => {
  const directory = path.join(root, ".github/workflows");
  const expected = renderWorkflowCatalog(readWorkflowEntries(directory));
  assert.equal(readRepoFile(".github/workflows/README.md"), expected);
  const headingAnchors = new Set(
    [...expected.matchAll(/^## (.+)$/gmu)].map((match) => githubHeadingAnchor(match[1]!)),
  );
  for (const match of expected.matchAll(/\]\(#([^)]+)\)/gmu)) {
    assert.ok(headingAnchors.has(match[1]!), `catalog link has no heading: #${match[1]}`);
  }
});

test("non-release workflows use maximum Blacksmith runner classes", () => {
  const workflows = path.join(root, ".github/workflows");
  const violations = fs
    .readdirSync(workflows)
    .filter((file) => /\.ya?ml$/.test(file) && workflowGroupId(file) !== "02")
    .flatMap((file) => {
      const source = readRepoFile(`.github/workflows/${file}`);
      return workflowRunnerLabels(source)
        .filter((runner) => !approvedRunnerLabels.has(runner))
        .map((runner) => `${file}: ${runner}`);
    });
  assert.deepEqual(violations, []);
});

test("runner policy parses multiline and matrix declarations", () => {
  const source = `
jobs:
  direct:
    runs-on:
      - blacksmith-32vcpu-ubuntu-2404
  matrix:
    strategy:
      matrix:
        include:
          - runner: blacksmith-32vcpu-ubuntu-2404
          - runner: blacksmith-8vcpu-ubuntu-2404
    runs-on: \${{ matrix.runner }}
`;
  assert.deepEqual(workflowRunnerLabels(source), [
    "blacksmith-32vcpu-ubuntu-2404",
    "blacksmith-32vcpu-ubuntu-2404",
    "blacksmith-8vcpu-ubuntu-2404",
  ]);
  assert.deepEqual(
    workflowRunnerLabels(source).filter((runner) => !approvedRunnerLabels.has(runner)),
    ["blacksmith-8vcpu-ubuntu-2404"],
  );
});

test("GitHub heading anchors preserve adjacent separators after punctuation removal", () => {
  assert.equal(
    githubHeadingAnchor("06 SFC, JSX, and compiler correctness"),
    "06-sfc-jsx-and-compiler-correctness",
  );
  assert.equal(githubHeadingAnchor("A & B"), "a--b");
});

test("the extracted docs readiness driver preserves TAP totals", () => {
  assert.deepEqual(
    parseTapCounts("# tests 8\n# pass 8\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n"),
    { tests: 8, pass: 8, fail: 0, cancelled: 0, skipped: 0, todo: 0 },
  );
});

test("embedded workflow programs only shrink", () => {
  const debt = JSON.parse(readRepoFile("tools/config/workflow-inline-script-debt.json")) as Record<
    string,
    number
  >;
  const current: Record<string, number> = {};
  for (const directory of [".github/workflows", ".github/actions"]) {
    const absolute = path.join(root, directory);
    for (const entry of fs.readdirSync(absolute, { recursive: true })) {
      if (typeof entry !== "string" || !/\.ya?ml$/.test(entry)) continue;
      const relative = path.join(directory, entry);
      const count = [...readRepoFile(relative).matchAll(embeddedProgram)].length;
      if (count > 0) current[relative] = count;
    }
  }
  for (const [file, count] of Object.entries(current)) {
    assert.ok(file in debt, `${file} introduced an embedded program; use a typed tools/ driver`);
    assert.ok(
      count <= debt[file]!,
      `${file} increased embedded programs from ${debt[file]} to ${count}`,
    );
  }
});
