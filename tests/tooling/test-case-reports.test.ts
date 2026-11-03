import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { TEST_AREAS, areaFor } from "../../tools/support/compat/github/test-areas.ts";
import {
  nextestCases,
  readPackets,
  workspaceCrates,
} from "../../tools/support/compat/github/test-cases.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

const crates = workspaceCrates(root);

test("nextest cases keep their final outcome, crate path and decoded name", () => {
  const junit = `<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run" tests="6">
  <testsuite name="vize_canon" tests="5">
    <testcase name="passes" classname="vize_canon" time="0.1"/>
    <testcase name="fails" classname="vize_canon" time="0.1"><failure type="test failure"><![CDATA[
      <testcase name="ghost" classname="vize_canon"/>]]></failure><system-out><![CDATA[out]]></system-out></testcase>
    <testcase name="errors" classname="vize_canon"><error type="timeout"/></testcase>
    <testcase name="ignored" classname="vize_canon"><skipped/></testcase>
    <testcase name="retried&lt;T&gt;" classname="vize_canon"><flakyFailure type="test failure"/></testcase>
  </testsuite>
  <testsuite name="vize::integration" tests="1">
    <testcase name="lint_x::y" classname="vize::integration"/>
  </testsuite>
</testsuites>`;
  assert.deepEqual(nextestCases(junit, crates), [
    { path: "crates/vize_canon", name: "vize_canon passes", outcome: "passed" },
    { path: "crates/vize_canon", name: "vize_canon fails", outcome: "failed" },
    { path: "crates/vize_canon", name: "vize_canon errors", outcome: "failed" },
    { path: "crates/vize_canon", name: "vize_canon ignored", outcome: "skipped" },
    // A retried pass stays passed.
    { path: "crates/vize_canon", name: "vize_canon retried<T>", outcome: "passed" },
    { path: "crates/vize", name: "vize::integration lint_x::y", outcome: "passed" },
  ]);
  assert.throws(
    () => nextestCases(`<testcase name="x" classname="not_a_member"/>`, crates),
    /not_a_member is not a workspace member/,
  );
  assert.throws(() => nextestCases(`<testcase name="x"/>`, crates), /no classname or name/);
});

test("every workspace crate belongs to a named product area", () => {
  assert.equal(crates.get("vize"), "crates/vize");
  assert.equal(crates.get("vize_benchmarks"), "tools/benchmarks/crates/vize");
  for (const [crate, member] of crates)
    assert.doesNotMatch(areaFor(member, `${crate} t`), /^Other/, `${crate} at ${member}`);
  assert.equal(areaFor("tests/tooling/github-comments.test.ts", "t"), "Tooling");
  assert.equal(areaFor("crates/vize_croquis", "t"), "Compiler");
  assert.equal(areaFor("crates/vize_croquis_cf", "t"), "Type checker");
  assert.equal(areaFor("tests/native/css.mjs", "t"), "Other (tests/native)");
  // CLI integration modules follow the product they exercise.
  const cli = (name: string) => areaFor("crates/vize", `vize::integration ${name}`);
  assert.deepEqual(
    ["check_a::b", "typecheck_a", "lsp_a", "lint_a", "fmt_a", "format_a", "build_a"].map(cli),
    ["Type checker", "Type checker", "LSP", "Linter", "Formatter", "Formatter", "CLI"],
  );
  assert.equal(areaFor("crates/vize", "vize commands::lint::tests::x"), "CLI");
  const owners = TEST_AREAS.flatMap(({ paths }) => paths);
  for (const owner of owners)
    for (const other of owners)
      assert.ok(owner === other || !other.startsWith(`${owner}/`), `${other} nests in ${owner}`);
  assert.equal(new Set(owners).size, owners.length);
});

function withDirectory(check: (directory: string) => void) {
  return () => {
    const directory = mkdtempSync(path.join(tmpdir(), "vize-test-cases-"));
    try {
      check(directory);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  };
}

void test(
  "the node test reporter records what Node counts as tests, load failures included",
  withDirectory((directory) => {
    writeFileSync(
      path.join(directory, "a.test.ts"),
      `import { describe, test } from "node:test";
test("passes", () => {});
test("fails", () => { throw new Error("no"); });
test("skips", { skip: true }, () => {});
test("todo", { todo: true }, () => {});
describe("group", () => { test("inner", () => {}); });
`,
    );
    writeFileSync(path.join(directory, "b.test.ts"), `throw new Error("load");\n`);
    const out = path.join(directory, "packet.json");
    const reporter = path.join(root, "tools/support/compat/github/test-case-reporter.ts");
    // A nested run must not look like a subtest of this one.
    const { NODE_TEST_CONTEXT: _context, ...nested } = process.env;
    const result = spawnSync(
      process.execPath,
      [`--test-reporter=${reporter}`, `--test-reporter-destination=${out}`, "--test"].concat([
        "a.test.ts",
        "b.test.ts",
      ]),
      {
        cwd: directory,
        encoding: "utf8",
        env: { ...nested, TEST_CASES_JOB: "Affected tooling (x)", GITHUB_RUN_ATTEMPT: "3" },
      },
    );
    assert.equal(result.status, 1, result.stderr);
    const packet = JSON.parse(readFileSync(out, "utf8")) as Record<string, unknown>;
    assert.deepEqual(packet, {
      version: 1,
      job: "Affected tooling (x)",
      attempt: 3,
      cases: [
        { path: "a.test.ts", name: "a.test.ts › passes", outcome: "passed" },
        { path: "a.test.ts", name: "a.test.ts › fails", outcome: "failed" },
        { path: "a.test.ts", name: "a.test.ts › skips", outcome: "skipped" },
        { path: "a.test.ts", name: "a.test.ts › todo", outcome: "skipped" },
        { path: "a.test.ts", name: "a.test.ts › inner", outcome: "passed" },
        { path: "b.test.ts", name: "b.test.ts", outcome: "failed" },
      ],
    });
  }),
);

void test(
  "each job's latest attempt is read and contradictory packets are refused",
  withDirectory((directory) => {
    const write = (file: string, job: string, attempt: number) => {
      mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
      const cases = [{ path: "crates/vize", name: `${job} ${attempt}`, outcome: "passed" }];
      writeFileSync(
        path.join(directory, file),
        JSON.stringify({ version: 1, job, attempt, cases }),
      );
    };
    write("a/one.json", "Rust tests (A)", 1);
    write("b/two.json", "Rust tests (A)", 2);
    write("three.json", "Rust tests (B)", 1);
    const read = readPackets(directory, 2)
      .map((packet) => [packet.job, packet.attempt])
      .sort(([left], [right]) => String(left).localeCompare(String(right)));
    assert.deepEqual(read, [
      ["Rust tests (A)", 2],
      ["Rust tests (B)", 1],
    ]);
    assert.throws(() => readPackets(directory, 1), /later attempt 2/);
    write("c/again.json", "Rust tests (B)", 1);
    assert.throws(() => readPackets(directory, 2), /Two packets for Rust tests \(B\) 1/);
  }),
);

type Step = { id?: string; name?: string; if?: string; uses?: string; run?: string };
type Job = {
  name?: string;
  env?: Record<string, string>;
  steps: Array<Step & { env?: Record<string, string>; with?: Record<string, string> }>;
};
const workflow = (file: string) =>
  parse(readRepoFile(".github", "workflows", file)) as { jobs: Record<string, Job> };

test("each test job names its case packet after itself and the report downloads them all", () => {
  const consumer = workflow("check.yml").jobs["test-report"].steps.find(
    (step) => step.id === "cases",
  );
  assert.deepEqual(consumer?.with, {
    pattern: "test-cases-*",
    path: "${{ runner.temp }}/test-cases",
  });

  const rust = workflow("pr-rust-products.yml").jobs["pr-rust-shard"];
  const record = rust.steps.find((step) => step.id === "cases");
  assert.equal(record?.env?.TEST_CASES_JOB, rust.name);
  assert.equal(rust.name, "Rust tests (${{ matrix.name }})");
  const rustUpload = rust.steps[rust.steps.indexOf(record ?? {}) + 1];
  assert.deepEqual(
    rustUpload.with?.name,
    "test-cases-rust-${{ matrix.cohort }}-${{ github.run_attempt }}",
  );
  assert.ok(record?.run?.endsWith(`"$RUNNER_TEMP/test-cases-rust-$COHORT.json"`));
  assert.equal(
    rustUpload.with?.path,
    "${{ runner.temp }}/test-cases-rust-${{ matrix.cohort }}.json",
  );

  const tooling = workflow("pr-source-checks.yml").jobs["affected-tooling"];
  const action = tooling.steps.find(
    (step) => step.uses === "./.github/actions/test-affected-tooling",
  );
  assert.equal(action?.with?.job, tooling.name);
  const composite = parse(
    readRepoFile(".github", "actions", "test-affected-tooling", "action.yml"),
  ) as {
    runs: Job;
  };
  const steps = composite.runs.steps;
  const tests = steps.find((step) => step.id === "tests");
  const upload = steps[steps.indexOf(tests ?? {}) + 1];
  assert.equal(tests?.env?.TEST_CASES_JOB, "${{ inputs.job }}");
  assert.equal(tests?.env?.VIZE_TEST_CASES, upload.with?.path);
  assert.equal(
    upload.with?.name,
    "test-cases-tooling-${{ inputs.shard }}-${{ github.run_attempt }}",
  );
  assert.equal(upload.if, "${{ !cancelled() && steps.tests.outcome != 'skipped' }}");
});
