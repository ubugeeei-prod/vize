import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  aggregateCheckNeedsResults,
  aggregateNeedsResults,
} from "../../tools/support/compat/github/require-needs-success.ts";
import type { GateReport } from "../../tools/support/compat/github/require-needs-success.ts";
import { requireRustTier } from "../../tools/support/compat/github/require-rust-tier.ts";

type Packet = { result: GateReport } | { error: { name: string; message: string } };
type Case = {
  name: string;
  mode: "needs" | "check" | "rust";
  event?: unknown;
  runRust?: unknown;
  needs: unknown;
  skippable?: Record<string, unknown>;
  expected: Packet;
};
const root = fileURLToPath(new URL("../../", import.meta.url));
const checker = join(root, "tools/support/typescript/check-project.ts");
const project = join(root, "tsconfig.ci-gates.json");
const ledger = readFileSync(new URL("./fixtures/ci-gates/packets.json", import.meta.url), "utf8");
const cases = JSON.parse(ledger) as Case[];
const artifact = join(root, "target/typescript-ci-gates", `node-${process.versions.node}`);
// #9357 retired WIT; the original ledger remains a whole historical witness.
const currentCheckPackets = new Map<string, Packet>([
  [
    "check-pr-queue-only-skip",
    {
      result: {
        exitCode: 1,
        message:
          "test-report gate: 1 of 3 needed jobs did not succeed.\n  - wit-contracts: skipped\ntest-report is a required status check, so it must not pass while a job it aggregates is red.\nOnly these jobs may skip on a pull request: nuxt-module-qualification.",
      },
    },
  ],
  [
    "check-missing-wit",
    {
      error: {
        name: "Error",
        message: "Check needs context is missing nuxt-module-qualification",
      },
    },
  ],
]);

function preserve(name: string, value: unknown): void {
  mkdirSync(artifact, { recursive: true });
  writeFileSync(join(artifact, `${name}.json`), JSON.stringify(value, null, 2));
}

function observe(row: Case, gate = { aggregateNeedsResults, aggregateCheckNeedsResults }): Packet {
  try {
    const result =
      row.mode === "needs"
        ? gate.aggregateNeedsResults(row.needs, row.skippable)
        : row.mode === "check"
          ? gate.aggregateCheckNeedsResults(row.needs, row.event)
          : requireRustTier(row.event, row.runRust, row.needs);
    return { result };
  } catch (error) {
    if (!(error instanceof Error)) throw error;
    return { error: { name: error.name, message: error.message } };
  }
}

void test("CI gates retain independently authored whole result and refusal packets", () => {
  assert.equal(
    createHash("sha256").update(ledger).digest("hex"),
    "025feeec3f683b000661f61fc9f560d203652624b0f822b94f6c22baaf496545",
    "the complete archived ledger must remain unchanged",
  );
  const nonJsonCases: Array<Case & { inputRecipe: string }> = [
    {
      name: "check-primitive-symbol-preserves-template-refusal",
      mode: "check",
      needs: null,
      event: Symbol("unsupported"),
      inputRecipe: 'Symbol("unsupported")',
      expected: {
        error: { name: "TypeError", message: "Cannot convert a Symbol value to a string" },
      },
    },
    {
      name: "check-boxed-symbol-preserves-template-refusal",
      mode: "check",
      needs: null,
      event: Object(Symbol("unsupported")),
      inputRecipe: 'Object(Symbol("unsupported"))',
      expected: {
        error: { name: "TypeError", message: "Cannot convert a Symbol value to a string" },
      },
    },
  ];
  const observations = [...cases, ...nonJsonCases].map((row) => ({
    ...row,
    currentExpected: currentCheckPackets.get(row.name) ?? row.expected,
    actual: observe(row),
  }));
  preserve("whole-packets", {
    node: process.version,
    executable: process.execPath,
    ledger,
    observations,
  });
  for (const row of observations) {
    if (currentCheckPackets.has(row.name))
      assert.notDeepEqual(
        row.actual,
        row.expected,
        `${row.name}: refuse the obsolete WIT contract`,
      );
    assert.deepEqual(row.actual, row.currentExpected, row.name);
  }
});

void test("the sole retired WIT clause replays every complete archived Check packet", async () => {
  const source = readFileSync(
    join(root, "tools/support/compat/github/require-needs-success.ts"),
    "utf8",
  );
  const currentClause = 'const queueOnly = ["nuxt-module-qualification"];';
  const previousClause = 'const queueOnly = ["wit-contracts", "nuxt-module-qualification"];';
  assert.equal(source.split(currentClause).length, 2, "one exact retirement clause");
  const historical = source.replace(currentClause, previousClause);
  assert.equal(
    createHash("sha256").update(historical).digest("hex"),
    "2e8d99c057b2a184c16f4af4ee9eacfd8425afeee473bab2f19ccb0cd80456f0",
    "whole gate source before 07f5645623a8d26d8909a9a53f8636682ce2192c",
  );
  const directory = mkdtempSync(join(tmpdir(), "vize-ci-gate-history-"));
  try {
    const original = join(directory, "original-gate.ts");
    writeFileSync(original, historical);
    const gate: Pick<
      typeof import("../../tools/support/compat/github/require-needs-success.ts"),
      "aggregateNeedsResults" | "aggregateCheckNeedsResults"
    > = await import(pathToFileURL(original).href);
    const observations = cases
      .filter((row) => row.mode === "check")
      .map((row) => ({ ...row, actual: observe(row, gate) }));
    preserve("pre-wit-retirement-whole-packets", { historical, observations });
    assert.equal(observations.length, 6, "all original Check cases must execute");
    for (const row of observations) assert.deepEqual(row.actual, row.expected, row.name);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

void test("only Nuxt may skip outside the queue and no retired WIT result is required", () => {
  const needs = {
    "pr-source-plan": { result: "success" },
    "nuxt-module-qualification": { result: "skipped" },
  };
  const green: GateReport = {
    exitCode: 0,
    message:
      "test-report gate: all 2 needed jobs are accounted for. 1 succeeded; 1 skipped on a pull request by design: nuxt-module-qualification.",
  };
  for (const event of ["pull_request", "push", "schedule", "workflow_dispatch"])
    assert.deepEqual(aggregateCheckNeedsResults(needs, event), green, event);
  assert.deepEqual(aggregateCheckNeedsResults(needs, "merge_group"), {
    exitCode: 1,
    message:
      "test-report gate: 1 of 2 needed jobs did not succeed.\n  - nuxt-module-qualification: skipped\ntest-report is a required status check, so it must not pass while a job it aggregates is red.\nOnly these jobs may skip on a pull request: .",
  });
  assert.deepEqual(
    aggregateCheckNeedsResults(
      { ...needs, "nuxt-module-qualification": { result: "success" } },
      "merge_group",
    ),
    {
      exitCode: 0,
      message:
        "test-report gate: all 2 needed jobs are accounted for. 2 succeeded; 0 skipped on a pull request by design.",
    },
  );
});

void test("real Node directly executes both erasable TypeScript gate entrypoints", () => {
  const good =
    "test-report gate: all 1 needed jobs are accounted for. 1 succeeded; 0 skipped on a pull request by design.\n";
  const red =
    "test-report gate: 1 of 1 needed jobs did not succeed.\n  - check-js: failure\ntest-report is a required status check, so it must not pass while a job it aggregates is red.\nOnly these jobs may skip on a pull request: .\n";
  const rows = [
    {
      name: "needs-success",
      script: "require-needs-success.ts",
      env: { NEEDS_JSON: '{"check-js":{"result":"success"}}' },
      expected: { status: 0, stdout: good, stderr: "" },
    },
    {
      name: "needs-red",
      script: "require-needs-success.ts",
      env: { NEEDS_JSON: '{"check-js":{"result":"failure"}}' },
      expected: { status: 1, stdout: "", stderr: red },
    },
    {
      name: "needs-absent",
      script: "require-needs-success.ts",
      env: { NEEDS_JSON: "" },
      expected: {
        status: 1,
        stdout: "",
        stderr:
          "NEEDS_JSON is required: pass ${{ toJSON(needs) }} to tools/support/compat/github/require-needs-success.ts\n",
      },
    },
    {
      name: "rust-full-pr",
      script: "require-rust-tier.ts",
      env: {
        NEEDS_JSON: '{"pr-rust-build":{"result":"success"},"pr-rust-shard":{"result":"success"}}',
        GITHUB_EVENT_NAME: "pull_request",
        RUN_RUST: "true",
      },
      expected: {
        status: 0,
        stdout:
          "test-report gate: all 2 needed jobs are accounted for. 2 succeeded; 0 skipped on a pull request by design.\n",
        stderr: "",
      },
    },
    {
      name: "rust-red-pr",
      script: "require-rust-tier.ts",
      env: {
        NEEDS_JSON: '{"pr-rust-build":{"result":"success"},"pr-rust-shard":{"result":"failure"}}',
        GITHUB_EVENT_NAME: "pull_request",
        RUN_RUST: "true",
      },
      expected: {
        status: 1,
        stdout:
          "test-report gate: 1 of 2 needed jobs did not succeed.\n  - pr-rust-shard: failure\ntest-report is a required status check, so it must not pass while a job it aggregates is red.\nOnly these jobs may skip on a pull request: .\n",
        stderr: "",
      },
    },
  ];
  for (const row of rows) {
    const result = spawnSync(
      process.execPath,
      [join(root, "tools/support/compat/github", row.script)],
      { cwd: root, env: { ...process.env, ...row.env }, encoding: "utf8", shell: false },
    );
    const actual = { status: result.status, stdout: result.stdout, stderr: result.stderr };
    preserve(row.name, {
      node: process.version,
      executable: process.execPath,
      ...row,
      actual,
      error: result.error,
    });
    assert.equal(result.error, undefined);
    assert.deepEqual(actual, row.expected, row.name);
  }
});

void test("the Rust CLI rejects disabled or missing plans even when every job reports success", () => {
  const needs = JSON.stringify({
    "merge-rust-differential": { result: "success" },
    "pr-rust-build": { result: "success" },
    "pr-rust-shard": { result: "success" },
  });
  for (const event of ["pull_request", "merge_group"])
    for (const plan of ["false", "", "TRUE"]) {
      const result = spawnSync(
        process.execPath,
        [join(root, "tools/support/compat/github/require-rust-tier.ts")],
        {
          cwd: root,
          env: { ...process.env, GITHUB_EVENT_NAME: event, RUN_RUST: plan, NEEDS_JSON: needs },
          encoding: "utf8",
          shell: false,
        },
      );
      assert.equal(result.error, undefined);
      assert.notEqual(result.status, 0, `${event}: ${plan}`);
      assert.match(
        result.stderr,
        /Rust source checks must run for every pull request and merge group/,
      );
    }
});

void test("CI gate project independently enforces strict types and erasable syntax", () => {
  const actual = spawnSync(process.execPath, [checker, project], {
    encoding: "utf8",
    shell: false,
  });
  preserve("owned-native-project", {
    status: actual.status,
    stdout: actual.stdout,
    stderr: actual.stderr,
    error: actual.error,
  });
  assert.equal(actual.error, undefined);
  assert.equal(actual.status, 0, actual.stdout + actual.stderr);
  const directory = mkdtempSync(join(tmpdir(), "vize-ci-gate-types-"));
  const source = join(directory, "witness.ts");
  const config = join(directory, "tsconfig.json");
  writeFileSync(join(directory, "package.json"), '{"type":"module"}');
  writeFileSync(
    config,
    JSON.stringify({
      extends: project,
      compilerOptions: { types: [] },
      files: [source],
      include: [],
    }),
  );
  try {
    for (const [name, text, diagnostic] of [
      ["correct", "export const count: number = 1;\n", null],
      ["wrong-type", 'export const count: number = "wrong";\n', /TS2322/],
      ["implicit-any", "export function identity(value) { return value; }\n", /TS7006/],
      ["nonerasable", "export enum Gate { Ready }\n", /TS1294/],
    ] as const) {
      writeFileSync(source, text);
      const result = spawnSync(process.execPath, [checker, config], {
        encoding: "utf8",
        shell: false,
      });
      preserve(`native-${name}`, {
        text,
        status: result.status,
        stdout: result.stdout,
        stderr: result.stderr,
        error: result.error,
      });
      assert.equal(result.error, undefined);
      if (diagnostic) {
        assert.notEqual(result.status, 0);
        assert.match(result.stdout + result.stderr, diagnostic);
      } else assert.equal(result.status, 0, result.stdout + result.stderr);
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
