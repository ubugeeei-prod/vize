import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  validateAutomationSteps,
  automationProgramReferences,
} from "../../tools/support/ci/automation-commands.ts";

import {
  githubHeadingAnchor,
  readWorkflowEntries,
  renderWorkflowCatalog,
} from "../../tools/ci/commands/workflow-catalog.ts";
import {
  legacyWorkflowNames,
  workflowNameViolations,
} from "../../tools/support/ci/workflow-names.ts";
import { parseTapCounts } from "../../tools/support/github/docs-preview-readiness.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";
import { plannedBusyRunner, plannedRunnerCondition } from "./support/planned-runner.ts";
import { withoutGeneratedHeader } from "../../tools/support/ci/generated-header.ts";
import { sourceRunnerChanges } from "./support/source-validation-runners.ts";

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
  // WSL2 needs nested virtualization, which Blacksmith Windows runners lack
  // (HCS ERROR_NOT_SUPPORTED when registering the distribution).
  "windows-2025",
]);

// Zero-compute plan and report jobs run on hosted Ubuntu so they never wait
// behind 32-vCPU build jobs at the Blacksmith concurrency cap (see #10975).
// They only read plans, gate `needs`, or publish reports: no Rust toolchain,
// sticky disk or build cache.
const hostedZeroComputeJobs: Readonly<Record<string, readonly string[]>> = {
  "check.yml": ["iteration-plan", "test-report"],
  "01-rust-feature-tests.yml": ["rust-feature-plan"],
  "06-browser-oracles.yml": ["changes"],
  "parcel-transformer.yml": ["native-plugin-matrix", "native-plugin-pack", "native-plugin-publish"],
  "pr-rust-checks.yml": ["rust-source-report"],
  "pr-rust-products.yml": ["rust-product-report"],
  "pr-source-checks.yml": ["pr-source-plan", "affected-tooling-report", "source-report"],
};
const hostedZeroComputeRunner = "ubuntu-24.04";

// Build-free planner and policy jobs take the smallest Blacksmith class: they
// finish in seconds, and a 32-vCPU slot spent on them delays a build job at
// the pool's concurrency cap. No Rust toolchain, sticky disk or build cache.
const smallRunnerJobs: Readonly<Record<string, readonly string[]>> = {
  "07-level-dependencies.yml": ["level-dependencies"],
  "08-nuxt-styles.yml": ["plan"],
  "08-vite-plus-types.yml": ["real-vite-plugin-types"],
  "10-musea-browser.yml": ["plan"],
  "11-zizmor.yml": ["plan", "zizmor"],
  "cleanup-obsolete-merge-groups.yml": ["cleanup"],
  "parcel-transformer.yml": ["packaging-changes"],
  "release-smoke-contracts.yml": ["bun-planner", "node-planner-baseline"],
  "title-policy.yml": ["issue-pr-title-policy"],
};
const smallRunner = "blacksmith-4vcpu-ubuntu-2404";

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

function workflowRunnerLabels(
  source: string,
  hostedJobs: readonly string[] = [],
  file = "",
  smallJobs: readonly string[] = [],
): string[] {
  const workflow = parse(source) as unknown;
  assert.ok(isRecord(workflow) && isRecord(workflow.jobs), "workflow must define jobs");
  return Object.entries(workflow.jobs).flatMap(([id, value]) => {
    assert.ok(isRecord(value), "workflow job must be a mapping");
    const sourceRunner = sourceRunnerChanges.find(
      (row) => row.workflow === `.github/workflows/${file}` && row.job === id,
    );
    if (sourceRunner && sourceRunner.after === value["runs-on"]) return [];
    if (file === "07-style-module-products.yml" && id === "nearest-line-starts") {
      // Kani's ARM64 binary requires glibc 2.39. The x86 byte search uses CPUID.
      assert.equal(value["runs-on"], "blacksmith-32vcpu-ubuntu-2404-arm", id);
      return [];
    }
    if (hostedJobs.includes(id)) {
      assert.equal(value["runs-on"], hostedZeroComputeRunner, id);
      return [];
    }
    if (smallJobs.includes(id)) {
      assert.equal(value["runs-on"], smallRunner, id);
      return [];
    }
    // A planner-selected skip runs hosted; the selected path is the busy runner.
    if (plannedRunnerCondition(file, id, value["runs-on"])) return [plannedBusyRunner];
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
        assert.match(strategy.matrix, /^\$\{\{\s*fromJSON\(.+\)(?:\.[\w-]+)*\s*\}\}$/u);
        return [];
      }
      assert.ok(resolved.length > 0, `cannot resolve runner matrix expression: ${runner}`);
      return resolved;
    });
  });
}

test("the flat GitHub workflow directory has a generated navigation catalog", () => {
  const expected = renderWorkflowCatalog(readWorkflowEntries(root));
  assert.equal(
    withoutGeneratedHeader(readRepoFile(".github/workflows/README.md"), "html"),
    expected,
  );
  const headingAnchors = new Set(
    [...expected.matchAll(/^## (.+)$/gmu)].map((match) => githubHeadingAnchor(match[1]!)),
  );
  for (const match of expected.matchAll(/\]\(#([^)]+)\)/gmu)) {
    assert.ok(headingAnchors.has(match[1]!), `catalog link has no heading: #${match[1]}`);
  }
});

test("non-release workflows use reviewed build, source validation and planner runners", () => {
  const violations = readWorkflowEntries(root)
    .filter(({ groupId }) => groupId !== "02")
    .flatMap(({ file }) => {
      const source = readRepoFile(`.github/workflows/${file}`);
      return workflowRunnerLabels(source, hostedZeroComputeJobs[file], file, smallRunnerJobs[file])
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

test("the validation pool accepts only exact reviewed source job selectors", () => {
  const missing = "jobs:\n  planner: {}\n";
  assert.throws(() => workflowRunnerLabels(missing, ["planner"], "unreviewed.yml"));
  assert.throws(() => workflowRunnerLabels(missing, [], "unreviewed.yml", ["planner"]));
  for (const row of sourceRunnerChanges) {
    const file = path.basename(row.workflow);
    const job = (id: string) => `jobs:\n  ${id}:\n    runs-on: ${row.after}\n`;
    assert.deepEqual(workflowRunnerLabels(job(row.job), [], file), []);
    for (const [source, filename] of [
      [job("unreviewed"), file],
      [job(row.job), "unreviewed.yml"],
    ])
      assert.throws(() => assert.deepEqual(workflowRunnerLabels(source!, [], filename), []));
  }
});

test("the line-start proof requires its Linux ARM64 runner", () => {
  const file = "07-style-module-products.yml";
  const runner = "blacksmith-32vcpu-ubuntu-2404-arm";
  const job = (id: string, label: string) => `jobs:\n  ${id}:\n    runs-on: ${label}\n`;
  assert.deepEqual(workflowRunnerLabels(job("nearest-line-starts", runner), [], file), []);
  assert.throws(
    () => workflowRunnerLabels(job("nearest-line-starts", "ubuntu-22.04-arm"), [], file),
    /nearest-line-starts/u,
  );
  for (const [filename, id] of [
    [file, "scss-products"],
    ["other.yml", "nearest-line-starts"],
  ] as const) {
    assert.deepEqual(workflowRunnerLabels(job(id, runner), [], filename), [runner]);
  }
  assert.equal(approvedRunnerLabels.has(runner), false);
});

test("only reviewed planner conditions may route a job to hosted Ubuntu", () => {
  const job = (runner: string) => `jobs:\n  security-audit:\n    runs-on: ${runner}\n`;
  const reviewed =
    "${{ needs.iteration-plan.outputs.dependencies == 'true' && 'blacksmith-32vcpu-ubuntu-2404' || 'ubuntu-24.04' }}";
  assert.deepEqual(workflowRunnerLabels(job(reviewed), [], "check.yml"), [plannedBusyRunner]);
  for (const [file, runner] of [
    ["other.yml", reviewed],
    ["check.yml", reviewed.replace("dependencies", "engines")],
  ])
    assert.deepEqual(
      workflowRunnerLabels(job(runner), [], file).filter(
        (label) => !approvedRunnerLabels.has(label),
      ),
      [runner],
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

test("every action and workflow step invokes a named program", () => {
  for (const directory of [".github/workflows", ".github/actions"]) {
    const absolute = path.join(root, directory);
    for (const entry of fs.readdirSync(absolute, { recursive: true, encoding: "utf8" })) {
      if (!/\.ya?ml$/u.test(entry)) continue;
      const relative = path.join(directory, entry);
      validateAutomationSteps(relative, parse(readRepoFile(relative)));
    }
  }
});

test("every named automation program is available in the same checkout as its caller", () => {
  function check(directory: string, source: string) {
    for (const name of automationProgramReferences(source)) {
      assert.match(name, /^(?:(?:\.\.\/){3,4}(?:tools\/ci\/)?lib\/scripts\/)?[\w.-]+$/u, name);
      const program = path.join(directory, name);
      assert.ok(fs.statSync(program, { throwIfNoEntry: false })?.isFile(), program);
    }
  }
  const workflows = path.join(root, ".github/workflows");
  for (const file of fs.readdirSync(workflows).filter((file) => file.endsWith(".yml"))) {
    const stem = path.basename(file, ".yml");
    const moved = path.join(root, "tools/ci/workflows", stem, "scripts");
    const directory = fs.existsSync(moved)
      ? moved
      : path.join(root, ".github/scripts/workflows", stem);
    check(directory, fs.readFileSync(path.join(workflows, file), "utf8"));
    if (fs.existsSync(directory))
      for (const program of fs.readdirSync(directory))
        check(directory, fs.readFileSync(path.join(directory, program), "utf8"));
  }
  const actions = path.join(root, ".github/actions");
  for (const entry of fs.readdirSync(actions, { recursive: true, withFileTypes: true })) {
    if (entry.name !== "action.yml") continue;
    const directory = path.join(entry.parentPath, "scripts");
    check(directory, fs.readFileSync(path.join(entry.parentPath, entry.name), "utf8"));
    if (fs.existsSync(directory))
      for (const program of fs.readdirSync(directory))
        check(directory, fs.readFileSync(path.join(directory, program), "utf8"));
  }
});

test("workflow files follow the group-prefixed naming rules", () => {
  const files = fs
    .readdirSync(path.join(root, ".github/workflows"))
    .filter((file) => file.endsWith(".yml"));
  assert.deepEqual(workflowNameViolations(files), []);
  for (const { file, groupId } of readWorkflowEntries(root)) {
    const prefix = /^(\d\d)-/u.exec(file)?.[1];
    if (prefix != null) assert.equal(groupId, prefix, file);
  }
});

test("workflow naming rules reject unprefixed, investigative and duplicate names", () => {
  const violations = workflowNameViolations([
    ...legacyWorkflowNames,
    "release.yml",
    "release-jsr.yml",
    "release-verify.yml",
    "release-cadence.yml",
    "release-npm-bootstrap.yml",
    "pkg-pr-new.yml",
    "published-fresh-project.yml",
    "parcel-transformer.yml",
    "deploy-docs.yml",
    "01-check.yml",
    "lsp-check.yml",
    "05-lsp-public-replay.yml",
    "06-template-alias-11822.yml",
    "03-compile-bench.yml",
    "11-sticky-disk.yml",
    "08-sticky-disk.yml",
    "08-deploy-docs.yml",
    "03-vue-benchmarks.yml",
  ]);
  assert.deepEqual(violations, [
    "lsp-check.yml: name must be NN-<subject>.yml",
    '05-lsp-public-replay.yml: "public" names an investigation',
    '05-lsp-public-replay.yml: "replay" names an investigation',
    '06-template-alias-11822.yml: "11822" is an issue or version number',
    '03-compile-bench.yml: group 03 already implies "bench"',
    "08-sticky-disk.yml: subject is already used by 11-sticky-disk.yml",
    "08-deploy-docs.yml: subject is already used by deploy-docs.yml",
  ]);
});
