import assert from "node:assert/strict";
import {
  spawnSync,
  type SpawnSyncOptions,
  type SpawnSyncOptionsWithBufferEncoding,
  type SpawnSyncOptionsWithStringEncoding,
  type SpawnSyncReturns,
} from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { readPackets } from "../../tools/support/compat/github/test-cases.ts";
import { withPreparedNative } from "../../npm/native/scripts/test-preparation.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";

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

function successfulSpawnRecorder(calls: Array<{ command: string; args: readonly string[] }>) {
  function run(command: string): SpawnSyncReturns<NonSharedBuffer>;
  function run(
    command: string,
    options: SpawnSyncOptionsWithStringEncoding,
  ): SpawnSyncReturns<string>;
  function run(
    command: string,
    options: SpawnSyncOptionsWithBufferEncoding,
  ): SpawnSyncReturns<NonSharedBuffer>;
  function run(
    command: string,
    options?: SpawnSyncOptions,
  ): SpawnSyncReturns<string | NonSharedBuffer>;
  function run(command: string, args: readonly string[]): SpawnSyncReturns<NonSharedBuffer>;
  function run(
    command: string,
    args: readonly string[],
    options: SpawnSyncOptionsWithStringEncoding,
  ): SpawnSyncReturns<string>;
  function run(
    command: string,
    args: readonly string[],
    options: SpawnSyncOptionsWithBufferEncoding,
  ): SpawnSyncReturns<NonSharedBuffer>;
  function run(
    command: string,
    args?: readonly string[],
    options?: SpawnSyncOptions,
  ): SpawnSyncReturns<string | NonSharedBuffer>;
  function run(
    command: string,
    argsOrOptions?: readonly string[] | SpawnSyncOptions,
    options?: SpawnSyncOptions,
  ): SpawnSyncReturns<string | NonSharedBuffer> {
    const args: string[] = [];
    let selectedOptions = options;
    if (argsOrOptions !== undefined) {
      if ("length" in argsOrOptions) args.push(...argsOrOptions);
      else selectedOptions = argsOrOptions;
    }
    calls.push({ command, args });
    const encoding = selectedOptions?.encoding;
    const output = encoding && encoding !== "buffer" ? "" : Buffer.alloc(0);
    return {
      pid: 0,
      output: [null, output, output],
      stdout: output,
      stderr: output,
      status: 0,
      signal: null,
    };
  }
  return run;
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

  const assertNodeJob = (
    file: string,
    job: string,
    stepName: string,
    packet: string,
    artifact: string,
    jobName = job,
  ) => {
    const selected = workflow(file).jobs[job];
    const cases = selected.steps.find((step) => step.name === stepName);
    assert.equal(cases?.id, "cases");
    assert.equal(cases?.env?.TEST_CASES_JOB, selected.name ?? jobName);
    assert.ok(cases?.run?.includes("test-case-reporter.ts"));
    assert.ok(cases?.run?.includes(packet));
    const upload = selected.steps[selected.steps.indexOf(cases ?? {}) + 1];
    assert.equal(upload.with?.name, artifact);
    assert.ok(upload.with?.path?.endsWith(packet));
    assert.equal(upload.with?.["if-no-files-found"], "error");
  };
  assertNodeJob(
    "check.yml",
    "check-js",
    "Test fast JS/TS workflow contracts",
    "test-cases-check-js.json",
    "test-cases-check-js-${{ github.run_attempt }}",
  );
  assertNodeJob(
    "check.yml",
    "node-engine-compat",
    "Check package engine matrix",
    "test-cases-node-engine-${{ matrix.node-version }}.json",
    "test-cases-node-engine-${{ matrix.node-version }}-${{ github.run_attempt }}",
    "node-engine-compat (${{ matrix.node-version }})",
  );
  assertNodeJob(
    "pr-source-checks.yml",
    "ci-selection-tests",
    "Test CI selection when its inputs change",
    "test-cases-ci-selection.json",
    "test-cases-ci-selection-${{ github.run_attempt }}",
  );
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
          successfulSpawnRecorder(calls),
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
