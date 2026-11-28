import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { publish } from "./shipped-cli-nuxt-report.ts";
import { loadRetainedCampaign, verifyRetainedCampaign } from "./shipped-cli-nuxt-replay.ts";
import {
  assertInputs,
  balancedOrder,
  labels,
  loadSnapshot,
  relocate,
  safeRelative,
  sha256,
  validateOutput,
  validateProducer,
} from "./shipped-cli-nuxt.ts";

void test("the captured workload keeps all declarations and the external dependency context", () => {
  const { receipt, snapshot } = loadSnapshot();
  assert.equal(receipt.reportedFileCount, 1003);
  assert.equal(receipt.programInputCount, 1107);
  const components = snapshot.files[".nuxt/components.d.ts"];
  assert.equal((components.match(/export const Scale\d{4}:/gu) ?? []).length, 1000);
  assert.ok(
    snapshot.files[".nuxt/tsconfig.json"].includes(
      "../../vize-p1-nuxt-final-consumer/node_modules",
    ),
  );
  assert.equal(snapshot.expectedOutput.errorCount, 0);
  assert.equal(snapshot.expectedOutput.warningCount, 0);
});

void test("only the two captured absolute roots relocate, reversibly", () => {
  const { receipt } = loadSnapshot();
  const source = `${receipt.originalProject}/.nuxt ${receipt.originalDependencies}/node_modules ../../vize-p1-nuxt-final-consumer/node_modules untouched`;
  const moved = relocate(source, receipt, "/runner/work");
  assert.equal(
    moved,
    "/runner/work/vize-p1-nuxt-scale-consumer/.nuxt /runner/work/vize-p1-nuxt-final-consumer/node_modules ../../vize-p1-nuxt-final-consumer/node_modules untouched",
  );
  assert.equal(relocate(moved, receipt, "/runner/work", true), source);
});

void test("complete output oracle refuses missing programs, changed options, or a diagnostic", () => {
  const { receipt, snapshot } = loadSnapshot();
  const actual = JSON.parse(
    relocate(JSON.stringify(snapshot.expectedOutput), receipt, "/runner/work"),
  );
  assert.doesNotThrow(() => validateOutput(actual, snapshot, receipt, "/runner/work"));
  for (const mutate of [
    (j: typeof actual) => {
      j.programs = [];
    },
    (j: typeof actual) => {
      j.programs[0].compilerOptions.strict = false;
    },
    (j: typeof actual) => {
      j.files[0].diagnostics = [{ message: "unexpected" }];
    },
    (j: typeof actual) => {
      j.programs[0].files.pop();
    },
  ]) {
    const changed = structuredClone(actual);
    mutate(changed);
    assert.throws(
      () => validateOutput(changed, snapshot, receipt, "/runner/work"),
      /complete JSON/,
    );
  }
});

void test("producer verification rejects source, status, artifact, and digest substitutions", () => {
  const { receipt } = loadSnapshot();
  const p = receipt.producer;
  const run = {
    id: p.runId,
    head_sha: p.driverSha,
    path: p.workflow,
    event: "workflow_dispatch",
    conclusion: "success",
    status: "completed",
  };
  const artifact = {
    id: p.artifactId,
    name: p.artifactName,
    expired: false,
    digest: `sha256:${p.artifactSha256}`,
    workflow_run: { id: p.runId, head_sha: p.driverSha },
  };
  assert.doesNotThrow(() => validateProducer(run, artifact, receipt));
  for (const changed of [
    { ...run, head_sha: "0".repeat(40) },
    { ...run, conclusion: "failure" },
    { ...run, event: "pull_request" },
  ])
    assert.throws(() => validateProducer(changed, artifact, receipt));
  for (const changed of [
    { ...artifact, id: p.artifactId + 1 },
    { ...artifact, expired: true },
    { ...artifact, digest: `sha256:${"0".repeat(64)}` },
  ])
    assert.throws(() => validateProducer(run, changed, receipt));
});

void test("input verification refuses changed files, missing inputs, and new files", () => {
  const work = mkdtempSync(join(tmpdir(), "shipped-cli-nuxt-contract-"));
  try {
    const { receipt, snapshot } = loadSnapshot();
    const project = join(work, "vize-p1-nuxt-scale-consumer");
    const deps = join(work, "vize-p1-nuxt-final-consumer");
    mkdirSync(project);
    mkdirSync(deps);
    writeFileSync(join(project, "app.vue"), "<template />\n");
    writeFileSync(join(deps, "package-lock.json"), "frozen-lock");
    const inputs = {
      ...snapshot,
      files: { "app.vue": "<template />\n" },
      programInputSha256: { "app.vue": sha256("<template />\n") },
    };
    const seal = { ...receipt, lockSha256: sha256("frozen-lock") };
    assert.doesNotThrow(() => assertInputs(work, seal, inputs));
    writeFileSync(join(project, "app.vue"), "changed");
    assert.throws(() => assertInputs(work, seal, inputs), /input changed/);
    writeFileSync(join(project, "app.vue"), "<template />\n");
    writeFileSync(join(project, "extra.vue"), "extra");
    assert.throws(() => assertInputs(work, seal, inputs), /inventory changed/);
    rmSync(join(project, "extra.vue"));
    rmSync(join(project, "app.vue"));
    assert.throws(() => assertInputs(work, seal, inputs), /inventory changed/);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

void test("the fixed schedule balances all independent builds and reverses the twin ordering", () => {
  const order = balancedOrder();
  assert.equal(order.length, 32);
  for (const label of labels) assert.equal(order.filter((value) => value === label).length, 8);
  for (let offset = 0; offset < 32; offset += 8)
    assert.deepEqual(order.slice(offset, offset + 8), [
      "base",
      "head",
      "head-twin",
      "base-twin",
      "head-twin",
      "base-twin",
      "base",
      "head",
    ]);
});

void test("snapshot extraction refuses traversal and absolute paths", () => {
  for (const path of ["../escape", "/escape", "a/../escape", "a\\escape", ""])
    assert.throws(() => safeRelative(path));
  assert.doesNotThrow(() => safeRelative(".nuxt/types/components.d.ts"));
});

void test("report publication persists the complete evidence result and the incomplete state", () => {
  const out = mkdtempSync(join(tmpdir(), "shipped-cli-nuxt-report-"));
  const previousSummary = process.env.GITHUB_STEP_SUMMARY;
  process.env.GITHUB_STEP_SUMMARY = join(out, "step-summary.md");
  try {
    const { receipt } = loadSnapshot();
    const records = balancedOrder().map((label, index) => ({
      index,
      label,
      elapsed: 1,
      user: 2,
      system: 1,
      maxRssKiB: 100,
      outputSha256: "a".repeat(64),
    }));
    publish(out, receipt, records, true);
    const result = JSON.parse(readFileSync(join(out, "result.json"), "utf8"));
    assert.equal(result.complete, true);
    assert.deepEqual(result.records, records);
    assert.match(
      readFileSync(join(out, "summary.md"), "utf8"),
      /equal in all 4 warmups and 32 measured runs/,
    );
    publish(out, receipt, [], false);
    assert.equal(JSON.parse(readFileSync(join(out, "result.json"), "utf8")).complete, false);
    assert.match(
      readFileSync(join(out, "summary.md"), "utf8"),
      /incomplete; no qualification credit/,
    );
  } finally {
    if (previousSummary === undefined) delete process.env.GITHUB_STEP_SUMMARY;
    else process.env.GITHUB_STEP_SUMMARY = previousSummary;
    rmSync(out, { recursive: true, force: true });
  }
});

void test("the original retained campaign verifies all source/image/input/output/time links", () => {
  const { campaign, snapshot, entries } = loadRetainedCampaign();
  const verified = verifyRetainedCampaign(entries, campaign, snapshot);
  assert.equal(verified.records.length, 32);
  assert.equal(new Set(verified.records.map((row) => row.outputSha256)).size, 1);
  for (const [file, change] of [
    [
      "00-base.stdout.json",
      (value: unknown) => {
        (value as { errorCount: number }).errorCount = 1;
      },
    ],
    [
      "progress.json",
      (value: unknown) => {
        (value as Array<{ elapsed: number }>)[0].elapsed += 1;
      },
    ],
    [
      "inputs-after.json",
      (value: unknown) => {
        (value as Record<string, string>)["nuxt.config.ts"] = "0".repeat(64);
      },
    ],
    [
      "images/head/identity.json",
      (value: unknown) => {
        (value as { source: string }).source = "0".repeat(40);
      },
    ],
  ] as const) {
    const altered = new Map(entries);
    const value = JSON.parse(altered.get(file)!.toString("utf8"));
    change(value);
    altered.set(file, Buffer.from(JSON.stringify(value)));
    assert.throws(() => verifyRetainedCampaign(altered, campaign, snapshot), file);
  }
  const missing = new Map(entries);
  missing.delete("31-head.stdout.json");
  assert.throws(
    () => verifyRetainedCampaign(missing, campaign, snapshot),
    /missing campaign record/,
  );
});
