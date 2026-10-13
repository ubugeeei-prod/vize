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

type Step = { name?: string; if?: string };
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

void test("every runner-1 step of the tooling job has a reserved, correctly gated lead entry", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
    jobs: Record<string, { steps: Step[] }>;
  };
  const steps = workflow.jobs["pr-tooling-scripts"].steps;
  const lead = steps.filter((step) => step.if?.includes("matrix.index == 1"));
  assert.deepEqual(
    lead.map((step) => step.name),
    TOOLING_LEAD_STEPS.map(({ step }) => step),
  );
  for (const { step, output } of TOOLING_LEAD_STEPS) {
    const gate = lead.find((entry) => entry.name === step)?.if ?? "";
    const outputs = [...gate.matchAll(/needs\.pr-source-plan\.outputs\.([\w-]+) == 'true'/g)]
      .map(([, name]) => name)
      .filter((name) => name !== "tooling");
    if (output === "always") {
      assert.deepEqual(outputs, [], step);
      assert.doesNotMatch(gate, /event_name/, step);
    } else if (output === "merge-only") {
      assert.deepEqual(outputs, [], step);
      assert.match(gate, /github\.event_name == 'merge_group' &&/, step);
    } else {
      assert.deepEqual(outputs, [output], step);
      assert.match(gate, /\(github\.event_name == 'merge_group' \|\| \(/, step);
    }
  }
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

void test("logged records, lead steps and merges refresh only what a run measured", () => {
  const line = (shard: string, files: Record<string, number>, tier = "pr") =>
    `2026-10-13T06:26:43.1234567Z \u001b[32m${TOOLING_TIMINGS_LOG_MARKER} ${JSON.stringify({ tier, shard, files })}\u001b[0m`;
  const log = ["noise", line("1/2", { "tests/tooling/a.test.ts": 1200 }), "ℹ pass 1"].join("\n");
  assert.deepEqual(parseToolingTimingLog(log), [
    { tier: "pr", shard: "1/2", files: { "tests/tooling/a.test.ts": 1200 } },
  ]);
  for (const bad of [
    line("1/2", { a: -1 }),
    line("1/2", {}, "nightly"),
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
