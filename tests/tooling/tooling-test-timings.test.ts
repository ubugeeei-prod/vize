import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import {
  MAX_TOOLING_SHARDS,
  TOOLING_LEAD_STEPS,
  selectToolingShard,
  toolingLeadSteps,
  toolingShardMatrix,
  toolingTestFiles,
  TOOLING_TIMINGS_LOG_MARKER,
  TOOLING_TIMINGS_PATH,
  validateToolingTimings,
} from "../../tools/support/compat/github/tooling-test-shards.ts";
import {
  leadStepDurations,
  measureToolingJobs,
  mergeToolingTimings,
  parseToolingTimingLog,
  parseToolingLeadLog,
} from "../../tools/support/compat/github/update-tooling-test-timings.ts";
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

type Step = { name?: string; if?: string; uses?: string };
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

void test("the full tooling caller retains shard coordinates and native lead-step timing identities", () => {
  const workflow = parse(checkWorkflowJobSources());
  const job = workflow.jobs["test-scripts"];
  assert.equal(job.name, "Tooling (${{ matrix.name }})");
  assert.equal(job.strategy.matrix, "${{ fromJSON(needs.tooling-plan.outputs.matrix) }}");
  const call = job.steps.find((step: Step) => step.uses === "./.github/actions/test-full-tooling");
  assert.deepEqual(call.with, {
    producer: "${{ toJSON(needs.full-source-cli.outputs) }}",
    shard: "${{ matrix.index }}",
    total: "${{ matrix.total }}",
  });
  const worker = parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml"));
  const lead = worker.runs.steps.filter((step: Step) =>
    step.uses?.startsWith("./.github/actions/test-native-"),
  );
  assert.deepEqual(
    lead.map((step: Step) => [step.name, step.uses]),
    [
      ["Prepare plugin runtime and test native JS setup", "./.github/actions/test-native-js-setup"],
      [
        "Capture original native attribute values and fixed fresh mounts",
        "./.github/actions/test-native-attribute-values-7502",
      ],
      [
        "Join original historical Vue documents to pinned whole goldens",
        "./.github/actions/test-native-historical-sfc-documents",
      ],
    ],
  );
  for (const step of lead) assert.equal(step.if, "${{ !cancelled() && inputs.shard == '1' }}");
  const at = (second: number) => `2026-10-13T08:45:${String(second).padStart(2, "0")}Z`;
  assert.deepEqual(
    leadStepDurations(
      lead.map((step: Step, index: number) => ({
        name: step.name!,
        conclusion: index ? "cancelled" : "failure",
        started_at: at(0),
        completed_at: at(5 + index),
      })),
    ),
    {
      "native-js-setup": 5000,
      "native-attribute-values-7502": 6000,
      "historical-sfc-documents": 7000,
    },
  );
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
    tiers: {
      pr: {},
      merge: { "tests/tooling/a.test.ts": 1200, "tests/tooling/b.test.ts": 99_999 },
    },
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
      "tests/tooling/b.test.ts": 99_999,
      "tests/tooling/c.test.ts": 7,
    },
  });
  assert.deepEqual(merged.leadSteps, { ...table.leadSteps, ...measured.leadSteps });
  assert.equal(merged.defaultMs, table.defaultMs);
});

void test("all current descriptive matrix cohorts retain success, failed and cancelled raw observations", () => {
  const plan = {
    tier: "merge" as const,
    tests: toolingTestFiles(),
    leadSteps: toolingLeadSteps("merge", {}),
  };
  const matrix = toolingShardMatrix(plan).include;
  assert.ok(matrix.length > 1 && matrix.length <= MAX_TOOLING_SHARDS);
  const logs: Record<number, string> = {};
  const expected: Record<string, number> = {};
  const nested = "../../../../../tmp/original/tests/tooling/misskey.test.ts";
  const jobs = matrix.map(({ name, index, total }) => {
    const files = Object.fromEntries(
      selectToolingShard(plan, `${index}/${total}`).map((file, offset) => [
        file,
        index * 1000 + offset,
      ]),
    );
    Object.assign(expected, files);
    if (index === 1) files[nested] = 27;
    logs[index] =
      `${TOOLING_TIMINGS_LOG_MARKER} ${JSON.stringify({ tier: "merge", shard: `${index}/${total}`, files })}`;
    return {
      id: index,
      name: `Check / Tooling (${name})`,
      conclusion: ["success", "failure", "cancelled"][index % 3],
    };
  });
  expected[nested] = 27;
  assert.deepEqual(measureToolingJobs(jobs, (id) => logs[id]).tiers, { pr: {}, merge: expected });
  for (const name of ["runtime captures", "no selected test files"]) {
    const record = `${TOOLING_TIMINGS_LOG_MARKER} ${JSON.stringify({ tier: "merge", shard: "1/1", files: {} })}`;
    assert.deepEqual(
      measureToolingJobs(
        [{ id: 1, name: `Tooling (${name})`, conclusion: "failure" }],
        () => record,
      ).tiers,
      { pr: {}, merge: {} },
    );
  }
  for (const name of [
    "unrelated",
    "Not Tooling (a)",
    "Tooling (a) extra",
    "PR tooling scripts (1/2) incomplete",
  ]) {
    assert.deepEqual(
      measureToolingJobs([{ id: 1, name, conclusion: "success" }], () => {
        throw new Error("unrelated job logs must not be read");
      }).tiers,
      { pr: {}, merge: {} },
    );
  }
  const raw = (
    shard = "1/2",
    tier = "merge",
    files: unknown = { "tests/tooling/a.test.ts": 12, "tests/tooling/b.test.ts": 21 },
  ) => `${TOOLING_TIMINGS_LOG_MARKER} ${JSON.stringify({ tier, shard, files })}`;
  for (const name of [
    "Tooling (a)",
    "Tooling (a + 2 files)",
    "Tooling (missing + 1 files)",
    "Tooling (a + 0 files)",
    "Tooling (1/2)",
    "Tooling (runtime captures)",
  ]) {
    assert.throws(
      () => measureToolingJobs([{ id: 1, name, conclusion: "failure" }], () => raw()),
      /cohort mismatch/,
    );
  }
  for (const shard of ["0/1", "2/1", "1/0", "01/2", "1/17", "1/2 extra", "1/9007199254740992"]) {
    assert.throws(() => parseToolingTimingLog(raw(shard)), /invalid tooling timing shard/);
    assert.throws(
      () =>
        measureToolingJobs(
          [{ id: 1, name: `PR tooling scripts (${shard})`, conclusion: "cancelled" }],
          () => raw(),
        ),
      /invalid tooling timing shard/,
    );
  }
  assert.throws(
    () => parseToolingTimingLog(raw("1/2", "merge", [])),
    /malformed tooling timing record/,
  );
  for (const extra of [raw("2/2"), raw("1/2", "pr")]) {
    assert.throws(
      () =>
        measureToolingJobs(
          [{ id: 1, name: "Tooling (a + 1 files)", conclusion: "success" }],
          () => `${raw()}\n${extra}`,
        ),
      /(?:shard|tier) mismatch/,
    );
  }
});

void test("raw composite capture spans preserve failures before file timing starts", () => {
  const first = "Prepare plugin runtime and test native JS setup";
  const second = "Capture original native attribute values and fixed fresh mounts";
  const historical = "Join original historical Vue documents to pinned whole goldens";
  const span = (name: string, id: string, outcome: string, ms: number | string) =>
    [
      `2026-10-16T17:13:11Z ##[start-action display=${name};id=${id}]`,
      `2026-10-16T17:13:12Z ##[end-action id=${id};outcome=${outcome};conclusion=${outcome};duration_ms=${ms}]`,
    ].join("\n");
  const log = [
    span(first, "native", "failure", 1234),
    span(`Post ${first}`, "post-native", "success", 25),
    span(second, "attribute", "cancelled", 42),
    span(historical, "historical", "failure", 37),
    span("unrelated setup", "other", "success", 99999),
  ].join("\n");
  const expected = {
    "native-js-setup": 1259,
    "native-attribute-values-7502": 42,
    "historical-sfc-documents": 37,
  };
  assert.deepEqual(
    parseToolingLeadLog(
      span(
        TOOLING_LEAD_STEPS.find(({ id }) => id === "historical-sfc-documents")!.step,
        "legacy-historical",
        "success",
        55,
      ),
    ),
    { "historical-sfc-documents": 55 },
  );
  assert.deepEqual(parseToolingLeadLog(log), expected);
  assert.deepEqual(
    measureToolingJobs(
      [
        {
          id: 1,
          name: "Tooling (runtime captures)",
          conclusion: "failure",
          steps: [
            {
              name: "Run ./.github/actions/test-full-tooling",
              conclusion: "failure",
              started_at: "2026-10-16T17:12:24Z",
              completed_at: "2026-10-16T17:18:00Z",
            },
          ],
        },
      ],
      () => log,
    ),
    { tiers: { pr: {}, merge: {} }, leadSteps: expected },
  );
  assert.deepEqual(parseToolingLeadLog(span(first, "skip", "skipped", 0)), {});
  assert.deepEqual(parseToolingLeadLog(span(`Post ${first}`, "orphan", "success", 9)), {});
  assert.deepEqual(parseToolingLeadLog(`##[start-action display=${first};id=incomplete]`), {});
  assert.throws(
    () => parseToolingLeadLog(span(first, "invalid", "failure", -1)),
    /invalid tooling lead duration/,
  );
  for (const conclusion of [null, "skipped"]) {
    assert.deepEqual(
      measureToolingJobs([{ id: 1, name: "Tooling (runtime captures)", conclusion }], () => {
        throw new Error("no completed job log");
      }),
      { tiers: { pr: {}, merge: {} }, leadSteps: {} },
    );
  }
});
