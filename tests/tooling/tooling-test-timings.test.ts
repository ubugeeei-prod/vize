import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import {
  TOOLING_LEAD_STEPS,
  TOOLING_TIMINGS_LOG_MARKER,
  TOOLING_TIMINGS_PATH,
  validateToolingTimings,
} from "../../tools/support/compat/github/tooling-test-shards.ts";
import {
  leadStepDurations,
  measureToolingJobs,
  mergeToolingTimings,
  parseToolingTimingLog,
} from "../../tools/support/compat/github/update-tooling-test-timings.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  uses?: string;
  run?: string;
  with?: Record<string, string>;
  env?: Record<string, string>;
};
const reporter = fileURLToPath(
  new URL("../../tools/support/compat/github/tooling-test-timing-reporter.ts", import.meta.url),
);
const committed = () =>
  validateToolingTimings(JSON.parse(readFileSync(TOOLING_TIMINGS_PATH, "utf8")));

void test("the committed timing table is complete, sorted and names only tooling tests", () => {
  const table = committed();
  assert.deepEqual(
    Object.keys(table.leadSteps),
    TOOLING_LEAD_STEPS.map(({ id }) => id),
  );
  for (const files of Object.values(table.tiers)) {
    const names = Object.keys(files);
    assert.ok(names.length > 0);
    assert.deepEqual(
      names,
      [...names].sort((left, right) => (left < right ? -1 : 1)),
    );
    for (const name of names)
      assert.match(name, /^tests\/tooling\/(?:[^/]+\/)?[^/]+\.test\.(?:ts|mjs)$/);
  }
  for (const invalid of [
    { ...table, schema: "other" },
    { ...table, defaultMs: 0 },
    { ...table, leadSteps: { ...table.leadSteps, extra: 1 } },
    { ...table, tiers: { pr: table.tiers.pr } },
    { ...table, tiers: { ...table.tiers, pr: { "tests/tooling/a.test.ts": -1 } } },
    { ...table, tiers: { ...table.tiers, merge: { "tests/tooling/a.test.ts": 1.5 } } },
  ]) {
    assert.throws(() => validateToolingTimings(invalid), /invalid tooling test timing table/);
  }
});

void test("scheduled tooling forwards its execution identity into the timing record", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as {
    jobs: Record<string, { steps: Step[] }>;
  };
  const worker = workflow.jobs["test-scripts"].steps.find(
    (step) => step.uses === "./.github/actions/test-full-tooling",
  );
  assert.equal(worker?.with?.shard, "${{ matrix.index }}");
  assert.equal(worker?.with?.total, "${{ matrix.total }}");
  const action = parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml")) as {
    runs: { steps: Step[] };
  };
  const tests = action.runs.steps.find(
    (step) => step.run === "vp run --workspace-root test:scripts:planned",
  );
  assert.equal(tests?.env?.VIZE_TOOLING_TEST_TIER, "merge");
  assert.equal(tests?.env?.VIZE_TOOLING_TEST_SHARD, "${{ inputs.shard }}/${{ inputs.total }}");
  assert.equal(tests?.env?.VIZE_TOOLING_TEST_PLAN, "${{ runner.temp }}/tooling-plan.json");
});

void test("the timing reporter records each file's wall time, imports included", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-tooling-timing-reporter-"));
  try {
    const sleep = (ms: number) => `await new Promise((done) => setTimeout(done, ${ms}));`;
    writeFileSync(
      join(cwd, "slow-import.test.mjs"),
      `import { test } from "node:test";\n${sleep(200)}\ntest("fast", () => {});\n`,
    );
    writeFileSync(
      join(cwd, "nested.test.mjs"),
      `import { describe, test } from "node:test";\ndescribe("suite", () => { test("slow", async () => { ${sleep(150)} }); });\n`,
    );
    writeFileSync(join(cwd, "empty.test.mjs"), `${sleep(50)}\n`);
    const destination = join(cwd, "timings.json");
    const env = { ...process.env };
    delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(
      process.execPath,
      [
        "--test-reporter=spec",
        "--test-reporter-destination=stdout",
        `--test-reporter=${reporter}`,
        `--test-reporter-destination=${destination}`,
        "--test",
        "--test-concurrency=1",
        "slow-import.test.mjs",
        "nested.test.mjs",
        "empty.test.mjs",
      ],
      { cwd, encoding: "utf8", env },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /✔ fast/);
    const files = JSON.parse(readFileSync(destination, "utf8"));
    assert.deepEqual(Object.keys(files).sort(), [
      "empty.test.mjs",
      "nested.test.mjs",
      "slow-import.test.mjs",
    ]);
    assert.ok(files["slow-import.test.mjs"] >= 200);
    assert.ok(files["nested.test.mjs"] >= 150);
    assert.ok(files["empty.test.mjs"] >= 50);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("historical numeric job records refresh only the files and lead steps they measured", () => {
  const line = (shard: string, files: Record<string, number>, tier = "pr") =>
    `2026-10-13T06:26:43.1234567Z \u001b[32m${TOOLING_TIMINGS_LOG_MARKER} ${JSON.stringify({ tier, shard, files })}\u001b[0m`;
  const log = ["noise", line("1/2", { "tests/tooling/a.test.ts": 1200 }), "ℹ pass 1"].join("\n");
  assert.deepEqual(parseToolingTimingLog(log), [
    { tier: "pr", shard: "1/2", files: { "tests/tooling/a.test.ts": 1200 } },
  ]);
  for (const bad of [
    line("1/2", { a: -1 }),
    line("1/2", {}, "nightly"),
    ...["0/16", "17/16", "1/0", "01/16", "1", "1/16 trailing"].map((shard) => line(shard, {})),
    `${TOOLING_TIMINGS_LOG_MARKER} {`,
  ]) {
    assert.throws(() => parseToolingTimingLog(bad));
  }

  const ssr = TOOLING_LEAD_STEPS[0].step;
  const css = TOOLING_LEAD_STEPS[1].step;
  const at = (second: number) => `2026-10-13T08:45:${String(second).padStart(2, "0")}Z`;
  const steps = [
    { name: ssr, conclusion: "success", started_at: at(0), completed_at: at(31) },
    { name: css, conclusion: "success", started_at: at(31), completed_at: at(50) },
    { name: `Post ${css}`, conclusion: "success", started_at: at(55), completed_at: at(59) },
    {
      name: TOOLING_LEAD_STEPS[3].step,
      conclusion: "skipped",
      started_at: at(0),
      completed_at: at(0),
    },
  ];
  assert.deepEqual(leadStepDurations(steps), { "native-ssr": 31000, "native-scoped-css": 23000 });

  const jobs = [
    { id: 1, name: "PR source checks / PR tooling scripts (1/2)", conclusion: "success", steps },
    { id: 2, name: "PR source checks / PR tooling scripts (2/2)", conclusion: "success", steps },
    { id: 3, name: "PR source checks / PR tooling scripts (2/2)", conclusion: "failure" },
    { id: 4, name: "PR source checks / Plan PR source checks", conclusion: "success" },
  ];
  const logs: Record<number, string> = {
    1: line("1/2", { "tests/tooling/a.test.ts": 1200 }, "merge"),
    2: line("2/2", { "tests/tooling/b.test.ts": 800 }, "merge"),
    3: line("2/2", { "tests/tooling/b.test.ts": 99_999 }, "merge"),
  };
  const measured = measureToolingJobs(jobs, (id) => logs[id] ?? "");
  assert.deepEqual(measured, {
    tiers: { pr: {}, merge: { "tests/tooling/a.test.ts": 1200, "tests/tooling/b.test.ts": 800 } },
    leadSteps: { "native-ssr": 31000, "native-scoped-css": 23000 },
  });
  assert.throws(
    () => measureToolingJobs([{ ...jobs[1], name: jobs[0].name }], (id) => logs[id]),
    /shard mismatch/,
  );

  const table = committed();
  const base = {
    ...table,
    tiers: {
      pr: { "tests/tooling/a.test.ts": 5, "tests/tooling/gone.test.ts": 5 },
      merge: { "tests/tooling/c.test.ts": 7, "tests/tooling/a.test.ts": 9 },
    },
  };
  const files = ["tests/tooling/a.test.ts", "tests/tooling/b.test.ts", "tests/tooling/c.test.ts"];
  const merged = mergeToolingTimings(base, measured, files);
  assert.deepEqual(merged.tiers, {
    pr: { "tests/tooling/a.test.ts": 5 },
    merge: {
      "tests/tooling/a.test.ts": 1200,
      "tests/tooling/b.test.ts": 800,
      "tests/tooling/c.test.ts": 7,
    },
  });
  assert.deepEqual(merged.leadSteps, { ...table.leadSteps, ...measured.leadSteps });
  assert.equal(merged.defaultMs, table.defaultMs);
});

void test("semantic workers consume logged identities and actual measurements, independent of labels", () => {
  // Retained excerpt from successful job 116928189349 in run 38887139884.
  // Its display label has no worker index; the emitted record is exactly 12/16.
  const job = {
    id: 116928189349,
    name: "Tooling (docs-render-concurrency-browser + 83 files)",
    conclusion: "success",
  };
  const files = {
    "tests/tooling/app-check-prepared-environment.test.ts": 235,
    "tests/tooling/docs-render-concurrency-browser.test.ts": 26353,
  };
  const log = `${TOOLING_TIMINGS_LOG_MARKER} ${JSON.stringify({ tier: "merge", shard: "12/16", files })}`;
  const requested: number[] = [];
  const measured = measureToolingJobs(
    [
      job,
      { id: 2, name: "Tooling (failed group)", conclusion: "failure" },
      { id: 3, name: "test-scripts", conclusion: "success" },
    ],
    (id) => {
      requested.push(id);
      return log;
    },
  );
  assert.deepEqual(requested, [job.id]);
  assert.deepEqual(measured, { tiers: { pr: {}, merge: files }, leadSteps: {} });
  assert.deepEqual(
    measureToolingJobs([{ ...job, name: `Check / ${job.name}` }], () => log),
    measured,
  );
  assert.throws(
    () => measureToolingJobs([job], () => `${log}\n${log.replace('"12/16"', '"13/16"')}`),
    /shard mismatch/,
  );
});

void test("semantic capture-only workers measure current lead steps without a fabricated timing record", () => {
  const at = (second: number) => `2026-10-16T10:30:${String(second).padStart(2, "0")}Z`;
  const steps = [
    {
      name: "Prepare plugin runtime and test native JS setup",
      conclusion: "success",
      started_at: at(0),
      completed_at: at(4),
    },
    {
      name: "Capture original native attribute values and fixed fresh mounts",
      conclusion: "success",
      started_at: at(4),
      completed_at: at(7),
    },
    {
      name: "Capture original historical Vue text and SFC documents",
      conclusion: "success",
      started_at: at(7),
      completed_at: at(12),
    },
    {
      name: "Post Capture original historical Vue text and SFC documents",
      conclusion: "success",
      started_at: at(12),
      completed_at: at(13),
    },
  ];
  const action = parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml")) as {
    runs: { steps: { name?: string }[] };
  };
  for (const step of steps.filter((step) => !step.name.startsWith("Post "))) {
    assert.ok(action.runs.steps.some(({ name }) => name === step.name));
  }
  assert.deepEqual(
    measureToolingJobs(
      [{ id: 1, name: "Tooling (runtime captures)", conclusion: "success", steps }],
      () => "No tooling tests selected by the declared inputs.",
    ),
    {
      tiers: { pr: {}, merge: {} },
      leadSteps: {
        "native-js-setup": 4000,
        "native-attribute-values-7502": 3000,
        "historical-sfc-documents": 6000,
      },
    },
  );
});
