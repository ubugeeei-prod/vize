import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import {
  jsPackageRoot,
  nodeTestArguments,
  plainNodeTestCase,
  vitestArguments,
  wrappedNodeArguments,
} from "../../tools/support/compat/github/js-test-cases.ts";
import { TEST_AREAS, areaFor } from "../../tools/support/compat/github/test-areas.ts";
import {
  nextestCases,
  nodeTestCases,
  vitestCases,
  workspaceCrates,
} from "../../tools/support/compat/github/test-cases.ts";
import { root } from "./support/github-workflows.ts";

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

test("node and vitest reports use the same case packet", () => {
  assert.equal(jsPackageRoot({}), root);
  assert.equal(jsPackageRoot({ VIZE_TEST_CASES_ROOT: "/tmp/cases/" }), "/tmp/cases");
  const ui = path.join(root, "npm/ui/src/one.test.ts");
  assert.deepEqual(
    nodeTestCases(
      [
        {
          type: "test:pass",
          data: { name: "passes", file: ui, details: { type: "test" } },
        },
        {
          type: "test:fail",
          data: { name: ui, file: ui, details: { type: "test" } },
        },
        { type: "test:pass", data: { name: "suite", file: ui, details: { type: "suite" } } },
      ],
      root,
    ),
    [
      {
        path: "npm/ui/src/one.test.ts",
        name: "npm/ui/src/one.test.ts › passes",
        outcome: "passed",
      },
      { path: "npm/ui/src/one.test.ts", name: "npm/ui/src/one.test.ts", outcome: "failed" },
    ],
  );
  assert.deepEqual(
    vitestCases(
      {
        testResults: [
          {
            name: ui,
            status: "failed",
            assertionResults: [
              {
                ancestorTitles: ["one.test.ts", "group"],
                title: "passes",
                status: "passed",
              },
              { ancestorTitles: [], title: "fails", status: "failed" },
              { ancestorTitles: ["group"], title: "later", status: "todo" },
              { ancestorTitles: ["group"], title: "holds", status: "pending" },
            ],
          },
          {
            name: path.join(root, "npm/builder/vite/src/load.test.ts"),
            status: "failed",
            assertionResults: [],
          },
        ],
      },
      root,
    ),
    [
      {
        path: "npm/ui/src/one.test.ts",
        name: "npm/ui/src/one.test.ts › group › passes",
        outcome: "passed",
      },
      { path: "npm/ui/src/one.test.ts", name: "npm/ui/src/one.test.ts › fails", outcome: "failed" },
      {
        path: "npm/ui/src/one.test.ts",
        name: "npm/ui/src/one.test.ts › group › later",
        outcome: "skipped",
      },
      {
        path: "npm/ui/src/one.test.ts",
        name: "npm/ui/src/one.test.ts › group › holds",
        outcome: "skipped",
      },
      {
        path: "npm/builder/vite/src/load.test.ts",
        name: "npm/builder/vite/src/load.test.ts",
        outcome: "failed",
      },
    ],
  );
  assert.throws(
    () =>
      vitestCases(
        {
          testResults: [
            { name: ui, assertionResults: [{ ancestorTitles: [], title: "x", status: "unknown" }] },
          ],
        },
        root,
      ),
    /status unknown/,
  );
  const reporter = path.join(root, "tools/support/compat/github/test-case-reporter.ts");
  assert.deepEqual(nodeTestArguments(["--test", "a.test.ts"], reporter, "/tmp/out.json"), [
    "--test",
    "--test-reporter=spec",
    "--test-reporter-destination=stdout",
    `--test-reporter=${reporter}`,
    "--test-reporter-destination=/tmp/out.json",
    "a.test.ts",
  ]);
  assert.deepEqual(
    nodeTestArguments(["--test-reporter=tap", "--test", "a.test.ts"], reporter, "/tmp/out.json"),
    [
      "--test-reporter=tap",
      "--test",
      `--test-reporter=${reporter}`,
      "--test-reporter-destination=/tmp/out.json",
      "a.test.ts",
    ],
  );
  assert.equal(nodeTestArguments(["plain.mjs", "--test"], reporter, "/tmp/out.json"), null);
  assert.deepEqual(
    nodeTestArguments(["--test", `--test-reporter=${reporter}`], reporter, "/tmp/out.json"),
    ["--test", `--test-reporter=${reporter}`],
  );
  assert.deepEqual(
    vitestArguments(["test", "run", "--config", "vite.config.ts"], "/tmp/out.json"),
    [
      "test",
      "run",
      "--reporter=default",
      "--reporter=json",
      "--outputFile.json=/tmp/out.json",
      "--config",
      "vite.config.ts",
    ],
  );
  assert.deepEqual(
    vitestArguments(["test", "run", "--reporter=dot", "a.test.ts"], "/tmp/out.json"),
    [
      "test",
      "run",
      "--reporter=json",
      "--outputFile.json=/tmp/out.json",
      "--reporter=dot",
      "a.test.ts",
    ],
  );
  assert.equal(vitestArguments(["test", "watch"], "/tmp/out.json"), null);
  assert.equal(vitestArguments(["pack"], "/tmp/out.json"), null);
  const vp = "/pkg/node_modules/vite-plus/bin/vp";
  const tsx = "/pkg/node_modules/tsx/dist/cli.mjs";
  assert.deepEqual(
    wrappedNodeArguments(
      [vp, "test", "run", "a.test.ts"],
      reporter,
      "/tmp/node.json",
      "/tmp/vitest.json",
    ),
    [
      vp,
      "test",
      "run",
      "--reporter=default",
      "--reporter=json",
      "--outputFile.json=/tmp/vitest.json",
      "a.test.ts",
    ],
  );
  assert.equal(
    wrappedNodeArguments(
      [vp, "exec", "tsx", "--test", "a.test.ts"],
      reporter,
      "/tmp/node.json",
      "/tmp/vitest.json",
    ),
    null,
  );
  assert.equal(
    wrappedNodeArguments(
      [vp, "test", "run", "--outputFile.json=/tmp/already.json"],
      reporter,
      "/tmp/node.json",
      "/tmp/vitest.json",
    ),
    null,
  );
  assert.deepEqual(
    wrappedNodeArguments(
      [tsx, "--test", "a.test.ts"],
      reporter,
      "/tmp/node.json",
      "/tmp/vitest.json",
    ),
    [
      tsx,
      "--test",
      "--test-reporter=spec",
      "--test-reporter-destination=stdout",
      `--test-reporter=${reporter}`,
      "--test-reporter-destination=/tmp/node.json",
      "a.test.ts",
    ],
  );
  assert.equal(
    wrappedNodeArguments([tsx, "src/file.ts"], reporter, "/tmp/node.json", "/tmp/vitest.json"),
    null,
  );
  assert.equal(
    wrappedNodeArguments(
      [tsx, "--test", `--test-reporter=${reporter}`, "a.test.ts"],
      reporter,
      "/tmp/node.json",
      "/tmp/vitest.json",
    ),
    null,
  );
});

test("plain Node assertion scripts produce command-level cases", () => {
  assert.deepEqual(plainNodeTestCase(["npm/oxlint/src/html-binding.test.mjs"], root, 0), {
    path: "npm/oxlint/src/html-binding.test.mjs",
    name: "npm/oxlint/src/html-binding.test.mjs › plain test command",
    outcome: "passed",
  });
  assert.equal(plainNodeTestCase(["npm/native/scripts/build-ci.mjs"], root, 0), null);
  assert.equal(plainNodeTestCase(["node_modules/dependency/src/hidden.test.ts"], root, 0), null);
  assert.equal(
    plainNodeTestCase(["tests/tooling/support/native-attribute-values-7502-judge.ts"], root, 0)
      ?.outcome,
    "passed",
  );
  assert.equal(plainNodeTestCase(["-e", "process.exit(0)"], root, 0), null);
  assert.equal(plainNodeTestCase(["--test", "src/file.test.ts"], root, 0), null);
  assert.equal(
    plainNodeTestCase(["npm/oxlint/src/html-binding.test.mjs"], root, 1)?.outcome,
    "failed",
  );
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
