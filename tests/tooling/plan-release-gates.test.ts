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
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

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
      { needs?: string[]; if?: string; uses?: string; with?: object; steps?: unknown[] }
    >;
  };
  const check = parse(checkWorkflowJobSources()) as Workflow;
  const workflow = parse(
    readRepoFile(".github", "workflows", "queue-release-gates.yml"),
  ) as Workflow;
  // #9198 moved this historical queue campaign to manual/scheduled qualification.
  // Current publication is guarded by exact-source full Check below.
  assert.equal(check.jobs["release-gates"], undefined);
  const gate = workflow.jobs["release-gates"]!;
  assert.deepEqual(
    [...gate.needs!].sort(),
    ["release-gate-plan", ...Object.values(RELEASE_GATE_JOBS)].sort(),
  );
  assert.deepEqual(
    workflow.jobs["vue-parity"]!.steps,
    check.jobs["vue-parity"]!.steps,
    "the queue runs the release vue-parity job unchanged",
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
