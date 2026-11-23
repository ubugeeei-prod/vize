import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

void test("Musea reports actual test-worker exit codes and signals without accepting failure", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "musea-worker-reporting-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const { scripts }: { scripts: { test: string } } = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  );
  assert.equal(scripts.test, "node ./scripts/package-tasks.cjs test");
  const require = createRequire(import.meta.url);
  type Command = { program: string; arguments: string[]; globs?: string[] };
  const { tasks } = require("../scripts/package-tasks.cjs") as { tasks: { test: Command[] } };
  const runners = tasks.test.filter(
    ({ program, arguments: args }) => program === "vp" && args[0] === "exec" && args[1] === "tsx",
  );
  assert.equal(runners.length, 1);
  const runner = runners[0]!;
  const firstFixture = runner.arguments.findIndex((argument) => runner.globs?.includes(argument));
  assert.ok(firstFixture > 2, "expected test options followed by worker file globs");
  const runnerOptions = runner.arguments.slice(2, firstFixture);
  const tsx = require.resolve("tsx/cli");
  const env: NodeJS.ProcessEnv = { ...process.env, FORCE_COLOR: "0" };
  delete env.NODE_TEST_CONTEXT;

  const workers = [
    {
      name: "exit",
      source: new URL("../tests/fixtures/exit-with-code.ts", import.meta.url),
      detail: /exitCode: 7/,
    },
    {
      name: "signal",
      source: new URL("../tests/fixtures/terminate-with-signal.ts", import.meta.url),
      detail: /signal: 'SIGTERM'/,
    },
  ];
  for (const worker of workers) {
    const fixture = path.join(directory, `${worker.name}.test.ts`);
    copyFileSync(worker.source, fixture);
    const result = spawnSync(process.execPath, [tsx, ...runnerOptions, fixture], {
      env,
      encoding: "utf8",
      timeout: 10_000,
      maxBuffer: 1024 * 1024,
    });
    assert.equal(result.error, undefined, `${worker.name}: ${result.error?.message}`);
    assert.equal(result.signal, null, `${worker.name}: outer runner was killed`);
    assert.equal(result.status, 1, `${worker.name}: worker failure must fail the runner`);
    assert.match(result.stdout, /test failed/);
    assert.match(result.stderr, /TAP version 13/);
    assert.match(result.stderr, /not ok 1/);
    assert.match(result.stderr, worker.detail);
    assert.match(result.stderr, /failureType: 'testCodeFailure'/);
    assert.match(result.stderr, /# fail 1/);
  }
});
