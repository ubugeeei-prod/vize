import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
  const invocation = scripts.test.split("vp exec tsx ")[1];
  assert.equal(typeof invocation, "string");
  const arguments_ = invocation.trim().split(/\s+/);
  const firstFixture = arguments_.findIndex((argument: string) => argument.includes("*"));
  assert.ok(firstFixture > 0);
  // Use the package's actual runner options, replacing only its file inventory
  // with controlled TypeScript workers that really exit or receive a signal.
  const runnerOptions = arguments_.slice(0, firstFixture);
  const tsx = createRequire(import.meta.url).resolve("tsx/cli");
  const env: NodeJS.ProcessEnv = { ...process.env, FORCE_COLOR: "0" };
  delete env.NODE_TEST_CONTEXT;

  const workers = [
    {
      name: "exit",
      source: "const code: number = 7;\nprocess.exit(code);\n",
      detail: /exitCode: 7/,
    },
    {
      name: "signal",
      source: 'const signal: NodeJS.Signals = "SIGTERM";\nprocess.kill(process.pid, signal);\n',
      detail: /signal: 'SIGTERM'/,
    },
  ];
  for (const worker of workers) {
    const fixture = path.join(directory, `${worker.name}.test.ts`);
    writeFileSync(fixture, worker.source);
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
