import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  jsPackageRoot,
  nodeTestArguments,
  vitestArguments,
  wrappedNodeArguments,
} from "../../tools/support/compat/github/js-test-cases.ts";
import { TEST_AREAS, areaFor } from "../../tools/support/compat/github/test-areas.ts";
import {
  nextestCases,
  nodeTestCases,
  readPackets,
  vitestCases,
  workspaceCrates,
} from "../../tools/support/compat/github/test-cases.ts";
import { withPreparedNative } from "../../npm/native/scripts/test-preparation.mjs";
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

  const packages = workflow("pr-js-checks.yml").jobs["pr-js-package-tests"];
  const packageTests = packages.steps.find((step) => step.name === "Test JS package group");
  assert.equal(packageTests?.id, "tests");
  assert.equal(packageTests?.env?.TEST_CASES_JOB, packages.name);
  assert.equal(packages.name, "PR JS package tests (${{ matrix.group }})");
  assert.equal(
    packageTests?.env?.VIZE_TEST_CASES,
    "${{ runner.temp }}/test-cases-js-${{ matrix.group }}.json",
  );
  const packageUpload = packages.steps[packages.steps.indexOf(packageTests ?? {}) + 1];
  assert.equal(
    packageUpload.with?.name,
    "test-cases-js-${{ matrix.group }}-${{ github.run_attempt }}",
  );
  assert.equal(packageUpload.with?.path, packageTests?.env?.VIZE_TEST_CASES);
  assert.equal(packageUpload.if, "${{ !cancelled() && steps.tests.outcome != 'skipped' }}");
  assert.equal(packageUpload.with?.["if-no-files-found"], "error");
});

void test(
  "package test preparation records node and vitest cases into one packet",
  withDirectory((directory) => {
    const calls: Array<{ command: string; args: readonly string[] }> = [];
    const saved = process.env.VIZE_TEST_CASES;
    process.env.VIZE_TEST_CASES = path.join(directory, "packet.json");
    try {
      assert.equal(
        withPreparedNative(
          path.join(directory, "missing-native"),
          "--native-test-owner=00000000-0000-0000-0000-000000000000",
          "vp",
          ["run", "test"],
          (command, args) => {
            calls.push({ command, args });
            return { status: 0 };
          },
        ),
        0,
      );
    } finally {
      if (saved === undefined) delete process.env.VIZE_TEST_CASES;
      else process.env.VIZE_TEST_CASES = saved;
    }
    assert.equal(calls.length, 1);
    assert.equal(calls[0].command, process.execPath);
    assert.equal(calls[0].args[1], "capture");
    assert.deepEqual(calls[0].args.slice(-4), ["--", "vp", "run", "test"]);

    writeFileSync(
      path.join(directory, "a.test.ts"),
      `import { test } from "node:test";
test("passes", () => {});
test("fails", () => { throw new Error("no"); });
`,
    );
    writeFileSync(
      path.join(directory, "b.test.ts"),
      `import { test } from "node:test";
test("passes", () => {});
`,
    );
    writeFileSync(path.join(directory, "plain.mjs"), "process.stdout.write('plain\\n');\n");
    const lib = path.join(directory, "lib");
    const tsxCli = path.join(directory, "tsx", "dist");
    const dotbin = path.join(directory, "node_modules", ".bin");
    mkdirSync(lib);
    mkdirSync(tsxCli, { recursive: true });
    mkdirSync(dotbin, { recursive: true });
    const vpCli = path.join(lib, "vp");
    writeFileSync(
      vpCli,
      `import { writeFileSync } from "node:fs";
const output = process.argv.find((arg) => arg.startsWith("--outputFile.json="))?.slice("--outputFile.json=".length);
if (!output) throw new Error("vitest json output was not requested");
writeFileSync(output, JSON.stringify({
  testResults: [{
    name: ${JSON.stringify(path.join(directory, "npm/ui/src/one.test.ts"))},
    status: "passed",
    assertionResults: [{ ancestorTitles: ["one.test.ts"], title: "renders", status: "passed" }],
  }],
}));
`,
    );
    writeFileSync(
      path.join(tsxCli, "cli.mjs"),
      `import { spawnSync } from "node:child_process";
const result = spawnSync(process.execPath, process.argv.slice(2), { stdio: "inherit", env: process.env });
process.exit(result.status ?? 1);
`,
    );
    writeFileSync(path.join(dotbin, "vp"), `#!/bin/sh\nexec node ${JSON.stringify(vpCli)} "$@"\n`, {
      mode: 0o755,
    });
    writeFileSync(
      path.join(dotbin, "tsx"),
      `#!/bin/sh\nexec node ${JSON.stringify(path.join(tsxCli, "cli.mjs"))} "$@"\n`,
      { mode: 0o755 },
    );
    const out = path.join(directory, "captured.json");
    const helper = path.join(root, "tools/support/compat/github/js-test-cases.ts");
    const { NODE_TEST_CONTEXT: _context, ...nested } = process.env;
    const result = spawnSync(
      process.execPath,
      [
        helper,
        "capture",
        "--",
        "bash",
        "-c",
        `export PATH=${JSON.stringify(dotbin)}:"$PATH"; node plain.mjs && node --test a.test.ts; status=$?; vp test run src/one.test.ts; tsx --test b.test.ts; exit $status`,
      ],
      {
        cwd: directory,
        encoding: "utf8",
        env: {
          ...nested,
          PATH: `${dotbin}${path.delimiter}${process.env.PATH ?? ""}`,
          TEST_CASES_JOB: "PR JS package tests (ui)",
          GITHUB_RUN_ATTEMPT: "4",
          VIZE_TEST_CASES: out,
          VIZE_TEST_CASES_ROOT: directory,
        },
      },
    );
    assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stdout, /plain/);
    const captured = JSON.parse(readFileSync(out, "utf8")) as {
      job: string;
      attempt: number;
      cases: Array<{ path: string; name: string; outcome: string }>;
    };
    assert.equal(captured.job, "PR JS package tests (ui)");
    assert.equal(captured.attempt, 4);
    assert.deepEqual(
      captured.cases
        .map((entry) => [entry.path, entry.name, entry.outcome])
        .sort((left, right) => left.join("\0").localeCompare(right.join("\0"))),
      [
        ["a.test.ts", "a.test.ts › fails", "failed"],
        ["a.test.ts", "a.test.ts › passes", "passed"],
        ["b.test.ts", "b.test.ts › passes", "passed"],
        ["npm/ui/src/one.test.ts", "npm/ui/src/one.test.ts › renders", "passed"],
      ],
    );
  }),
);
