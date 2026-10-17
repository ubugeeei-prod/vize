import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { brokenLinks } from "../../docs/scripts/verify-static-links.ts";
import {
  RELEASE_GATE_JOBS,
  checkReleaseGates,
  planReleaseGates,
} from "../../tools/support/compat/github/plan-release-gates.ts";
import { readRepoFile } from "./support/github-workflows.ts";
import {
  requiredReleaseWorkflows,
  requiredWorkflowJobNames,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";
import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";

const plan = (vueParity: boolean, docs: boolean, miri: boolean, fuzz: boolean) => ({
  "vue-parity": vueParity,
  docs,
  miri,
  fuzz,
});

void test("dormant queue planner retains every path-dependent release gate decision", () => {
  for (const [paths, expected] of [
    [["docs/content/ja/guide/cli.md"], plan(false, true, false, false)],
    [["docs/davinci/decisions/2026-10-14-x.md"], plan(false, false, false, false)],
    [["docs/davinci/plan/l2.md"], plan(true, true, true, true)],
    [["crates/vize_canon/src/lib.rs"], plan(true, false, true, true)],
    [
      ["tests/snapshots/check/__snapshots__/compiler-macros-check.snap"],
      plan(true, false, false, false),
    ],
    [["tests/fixtures/sfc/a.vue"], plan(true, false, false, true)],
    [["tests/fuzz/fuzz_targets/sfc_parse.rs"], plan(true, false, false, true)],
    [
      ["editors/vscode/src/extension.ts", "README.md", ".changeset/x.md"],
      plan(false, false, false, false),
    ],
    [["pnpm-lock.yaml"], plan(true, true, false, false)],
    [["npm/ui/src/button.ts"], plan(true, true, false, false)],
    [["playground/src/App.vue"], plan(false, true, false, false)],
    [[".github/workflows/check.yml"], plan(true, true, true, true)],
    [[], plan(true, true, true, true)],
  ] as const) {
    assert.deepEqual(planReleaseGates(paths), expected, paths.join(", "));
  }
});

const needs = (outputs: Record<string, string>, results: Record<string, string>) => ({
  "release-gate-plan": { result: "success", outputs },
  ...Object.fromEntries(
    Object.values(RELEASE_GATE_JOBS).map((job) => [job, { result: results[job] ?? "skipped" }]),
  ),
});
const off = { "vue-parity": "false", docs: "false", miri: "false", fuzz: "false" };

void test("dormant queue planner permits skips only for unselected gates", () => {
  assert.equal(checkReleaseGates(needs(off, {})).exitCode, 0);
  assert.equal(
    checkReleaseGates(needs({ ...off, docs: "true" }, { "docs-build": "success" })).exitCode,
    0,
  );
  assert.equal(checkReleaseGates(needs({ ...off, docs: "true" }, {})).exitCode, 1);
  assert.equal(
    checkReleaseGates(needs({ ...off, "vue-parity": "true" }, { "vue-parity": "failure" }))
      .exitCode,
    1,
  );
  assert.equal(checkReleaseGates(needs(off, { miri: "failure" })).exitCode, 1);
  assert.equal(checkReleaseGates(needs({ ...off, fuzz: "" }, {})).exitCode, 1);
  assert.equal(
    checkReleaseGates({ ...needs(off, {}), "release-gate-plan": { result: "failure" } }).exitCode,
    1,
  );
});

void test("dormant queue workflow retains each planned gate and its original predicate", () => {
  type Workflow = {
    jobs: Record<
      string,
      {
        needs?: string[];
        if?: string;
        uses?: string;
        with?: object;
        steps?: { uses?: string; run?: string }[];
      }
    >;
  };
  const check = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
  const workflow = parse(
    readRepoFile(".github", "workflows", "queue-release-gates.yml"),
  ) as Workflow;
  // #9198 moved this historical queue campaign to manual/scheduled qualification.
  // Current publication is guarded by exact-source full Check below.
  assert.equal(check.jobs["release-gates"], undefined);
  const gate = workflow.jobs["release-gates"]!;
  assert.equal(gate.if, "${{ always() }}");
  assert.ok(
    gate.steps?.some(
      (step) => step.run === "node tools/support/compat/github/plan-release-gates.ts --check",
    ),
  );
  assert.deepEqual(
    [...gate.needs!].sort(),
    ["release-gate-plan", ...Object.values(RELEASE_GATE_JOBS)].sort(),
  );
  assert.ok(
    workflow.jobs["vue-parity"]!.steps?.some(
      (step) => step.uses === "./.github/actions/check-vue-parity",
    ),
  );
  for (const [decision, job] of Object.entries(RELEASE_GATE_JOBS)) {
    assert.match(
      workflow.jobs[job]!.if!,
      new RegExp(
        `github\\.event_name == 'merge_group' && needs\\.release-gate-plan\\.outputs\\.${decision} == 'true'`,
      ),
      job,
    );
  }
  assert.equal(workflow.jobs["docs-build"]!.uses, "./.github/workflows/build-docs.yml");
  assert.deepEqual(workflow.jobs["docs-build"]!.with, { "links-only": true });
  assert.equal(workflow.jobs.miri!.uses, "./.github/workflows/miri.yml");
  assert.equal(workflow.jobs["fuzz-replay"]!.uses, "./.github/workflows/fuzz.yml");
  assert.deepEqual(workflow.jobs["fuzz-replay"]!.with, { mode: "replay" });
});

void test("publication requires the actual full Check aggregates and ordinary queue report", () => {
  type Job = {
    name?: string;
    needs?: string[];
    if?: string;
    steps?: Array<{ uses?: string; with?: Record<string, string>; run?: string }>;
  };
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as {
    jobs: Record<string, Job>;
  };
  const job = (key: string) => {
    const value = workflow.jobs[key];
    assert.ok(value, key);
    return value;
  };
  assert.deepEqual(requiredReleaseWorkflows, ["Check"]);
  const owners = {
    "test-scripts": "test-scripts-gate",
    "test-js-packages": "test-js-packages",
    "build-js-packages": "build-js-packages",
    "full-js-report": "full-js-report",
    "Full Rust workspace and doctests": "full-rust-workspace",
    "clippy-and-test": "clippy-and-test",
    "full-rust-report": "full-rust-report",
  };
  assert.deepEqual(requiredWorkflowJobNames("Check"), Object.keys(owners));
  for (const [name, owner] of Object.entries(owners)) {
    const actual = job(owner);
    assert.equal(actual.name ?? owner, name);
    assert.ok(actual.if);
    assert.match(actual.if, /github.event_name == 'workflow_dispatch'/);
  }
  for (const [owner, dependencies] of [
    ["test-scripts-gate", ["full-source-cli", "tooling-plan", "test-scripts"]],
    [
      "full-js-report",
      [
        "test-js-packages",
        "build-js-packages",
        "test-scripts",
        "full-source-cli",
        "check-js",
        "instruction-counts",
      ],
    ],
    ["full-rust-report", ["full-rust-workspace", "clippy-and-test"]],
  ] as const) {
    const report = job(owner);
    assert.deepEqual(report.needs, [...dependencies]);
    assert.ok(report.if);
    assert.match(report.if, /always\(\)/);
    assert.ok(report.steps);
    const context = Object.fromEntries(dependencies.map((name) => [name, { result: "success" }]));
    assert.equal(aggregateNeedsResults(context).exitCode, 0);
    for (const dependency of dependencies) {
      for (const result of ["skipped", "failure", "cancelled", "in_progress"]) {
        assert.equal(
          aggregateNeedsResults({ ...context, [dependency]: { result } }).exitCode,
          1,
          `${owner}:${dependency}:${result}`,
        );
      }
    }
    if (owner === "full-rust-report") {
      assert.ok(report.steps.some((step) => step.run?.includes("require-needs-success.ts")));
      assert.ok(
        report.steps.some((step) => step.run?.includes("rust-workspace-inventory.ts verify")),
      );
    } else {
      const call = report.steps.find(
        (step) => step.uses === "./.github/actions/report-full-js-qualification",
      );
      assert.ok(call);
      assert.deepEqual(call.with, { "needs-json": "${{ toJSON(needs) }}" });
    }
  }
  const workers = job("test-scripts") as Job & {
    strategy: { "fail-fast": boolean; matrix: string };
  };
  assert.equal(workers.strategy["fail-fast"], false);
  assert.equal(workers.strategy.matrix, "${{ fromJSON(needs.tooling-plan.outputs.matrix) }}");
  assert.deepEqual(workers.needs, ["full-source-cli", "tooling-plan"]);
  assert.ok(workers.steps);
  const worker = workers.steps.find((step) => step.uses === "./.github/actions/test-full-tooling");
  assert.ok(worker);
  assert.deepEqual(worker.with, {
    producer: "${{ toJSON(needs.full-source-cli.outputs) }}",
    shard: "${{ matrix.index }}",
    total: "${{ matrix.total }}",
  });
  const planner = readRepoFile(".github", "workflows", "full-tooling-plan.yml");
  assert.match(
    planner,
    /plan-tooling-tests.mjs 0000000000000000000000000000000000000000.*--tier merge/,
  );
  const ordinary = job("test-report");
  assert.ok(ordinary.needs);
  assert.ok(ordinary.needs.includes("pr-source-checks"));
  assert.ok(ordinary.if);
  assert.match(ordinary.if, /github.event_name == 'pull_request'/);
  assert.match(ordinary.if, /github.event_name == 'merge_group'/);
});

void test("the dormant queue docs review preserves the release link predicate without a browser", () => {
  const dist = mkdtempSync(join(tmpdir(), "vize-static-links-"));
  try {
    const page = (route: string, body: string) => {
      mkdirSync(join(dist, route), { recursive: true });
      writeFileSync(join(dist, route, "index.html"), body);
    };
    page(
      "guide/a",
      '<nav><a href="/missing/index.html">nav</a></nav><article class="content">' +
        '<h2 id="ok">x</h2><a href="#ok">a</a><a href="../b/index.html#%E3%81%82">b</a>' +
        '<a href="https://example.com/#x">external</a></article>',
    );
    page("guide/b", '<article class="content"><h2 id="あ">あ</h2></article>');
    assert.deepEqual(brokenLinks(dist, ["/guide/a", "/guide/b"]), []);
    page(
      "guide/c",
      '<article class="content"><a href="#gone">x</a><a href="../nope/">y</a></article>',
    );
    assert.deepEqual(brokenLinks(dist, ["/guide/c"]), [
      "/guide/c: missing anchor #gone",
      "/guide/c: broken link ../nope/",
    ]);
    assert.deepEqual(brokenLinks(dist, ["/guide/none"]), ["/guide/none: page was not built"]);
  } finally {
    rmSync(dist, { recursive: true, force: true });
  }
});
