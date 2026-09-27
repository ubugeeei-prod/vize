import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { requirePreparedNative, runViteTests } from "../../npm/builder/vite/scripts/run-tests.mjs";
import { testedPackages } from "../../tools/config/vite-plus/task-inputs.ts";
import { runInPackages, runTask } from "../../tools/config/vite-plus/task-commands.ts";
import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

void test("standalone Vite tests build first; prepared tests reuse the same native build", () => {
  const manifest = JSON.parse(readFileSync(join(root, "npm/builder/vite/package.json"), "utf8"));
  assert.equal(
    manifest.scripts["test:prepared"],
    "node scripts/run-tests.mjs --require-prepared && node src/test.ts && node --test src/vite-plus/*.test.ts",
  );
  for (const prepared of [false, true]) {
    const calls = [];
    const status = runViteTests(prepared, (command, args, options) => {
      calls.push({ command, args });
      assert.equal(options.env, process.env);
      assert.equal(options.cwd, join(root, "npm/builder/vite"));
      assert.equal(options.stdio, "inherit");
      return { status: 0 };
    });
    assert.equal(status, 0);
    assert.deepEqual(calls, [
      ...(prepared ? [] : [{ command: "pnpm", args: ["--dir", "../../native", "build:debug"] }]),
      { command: "pnpm", args: ["run", "test:prepared"] },
    ]);
  }
});

void test("native build failure prevents tests and package test failure remains a failure", () => {
  for (const status of [7, null]) {
    let calls = 0;
    assert.equal(
      runViteTests(false, () => {
        calls += 1;
        return { status };
      }),
      status ?? 1,
    );
    assert.equal(calls, 1);
  }
  assert.equal(
    runViteTests(true, () => ({ status: 3 })),
    3,
  );
});

void test("prepared tests require one loadable local addon without platform package fallback", () => {
  const nativeDir = mkdtempSync(join(tmpdir(), "vize-prepared-addon-"));
  const binding = join(nativeDir, "vize-vitrine.fixture.node");
  try {
    assert.throws(() => requirePreparedNative(nativeDir), /found 0.*build:native:test/);
    writeFileSync(binding, "invalid native addon");
    const loaded = [];
    requirePreparedNative(nativeDir, (file) => loaded.push(file));
    assert.deepEqual(loaded, [binding]);
    assert.throws(() => requirePreparedNative(nativeDir));
    writeFileSync(join(nativeDir, "vize-vitrine.other.node"), "second addon");
    assert.throws(() => requirePreparedNative(nativeDir), /found 2/);
  } finally {
    rmSync(nativeDir, { recursive: true, force: true });
  }
});

void test("root JS task preserves selection and ordering and only marks successful preparation", () => {
  const before = `${runTask("build:native:test")} && ${runInPackages("test", testedPackages, { concurrencyLimit: 1 })}`;
  const after = testAndBenchmarkTasks["test:js"].command;
  assert.equal(after.replace("VIZE_TEST_NATIVE_PREPARED=1 ", ""), before);
  const fixture = mkdtempSync(join(tmpdir(), "vize-prepared-root-"));
  const events = join(fixture, "events.jsonl");
  const vp = process.env.VIZE_VP_BIN ?? join(root, "node_modules/.bin/vp");
  try {
    writeFileSync(join(fixture, "package.json"), JSON.stringify({ name: "prepared-root" }));
    writeFileSync(join(fixture, "pnpm-workspace.yaml"), "packages:\n  - npm/**\n");
    writeFileSync(
      join(fixture, "prepare.mjs"),
      `import fs from 'node:fs'; fs.appendFileSync(${JSON.stringify(events)}, JSON.stringify({prepare:true})+'\\n'); process.exit(Number(process.env.VIZE_PREPARE_EXIT ?? 0));\n`,
    );
    for (const [index, pkg] of testedPackages.entries()) {
      const pkgDir = join(fixture, pkg);
      mkdirSync(pkgDir, { recursive: true });
      writeFileSync(
        join(pkgDir, "package.json"),
        JSON.stringify({ name: `prepared-${index}`, scripts: { test: "node probe.mjs" } }),
      );
      writeFileSync(
        join(pkgDir, "probe.mjs"),
        `import fs from 'node:fs'; fs.appendFileSync(${JSON.stringify(events)}, JSON.stringify({package:${JSON.stringify(pkg)},prepared:process.env.VIZE_TEST_NATIVE_PREPARED ?? null,env:process.env.VIZE_PREPARED_OTHER})+'\\n');\n`,
      );
    }
    const run = (command, preparationExit = 0) => {
      rmSync(events, { force: true });
      writeFileSync(
        join(fixture, "vite.config.mjs"),
        `export default { run: { tasks: { 'build:native:test': { command:'node prepare.mjs',cache:false },'test:js':{command:${JSON.stringify(command)},cache:false} } } };\n`,
      );
      const result = spawnSync(vp, ["run", "--workspace-root", "test:js"], {
        cwd: fixture,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${dirname(vp)}:${process.env.PATH}`,
          VIZE_PREPARE_EXIT: String(preparationExit),
          VIZE_PREPARED_OTHER: "same-environment",
          VIZE_TEST_NATIVE_PREPARED: "",
        },
      });
      return { result, records: readFileSync(events, "utf8").trim().split("\n").map(JSON.parse) };
    };
    const baseline = run(before);
    const prepared = run(after);
    for (const { result, records } of [baseline, prepared]) {
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      assert.deepEqual(records[0], { prepare: true });
      assert.equal(records.length, testedPackages.length + 1);
    }
    assert.deepEqual(
      prepared.records.slice(1),
      baseline.records.slice(1).map((record) => ({ ...record, prepared: "1" })),
    );
    const failed = run(after, 7);
    assert.notEqual(failed.result.status, 0);
    assert.deepEqual(failed.records, [{ prepare: true }]);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
