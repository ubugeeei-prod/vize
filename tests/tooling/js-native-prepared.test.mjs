import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  appendFileSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { requirePreparedNative, runViteTests } from "../../npm/builder/vite/scripts/run-tests.mjs";
import { testedPackages } from "../../tools/config/vite-plus/task-inputs.ts";
import { runInPackages, runTask } from "../../tools/config/vite-plus/task-commands.ts";
import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const helper = "npm/native/scripts/test-preparation.mjs";

function stageNative(fixture) {
  mkdirSync(join(fixture, "npm/native/scripts"), { recursive: true });
  copyFileSync(join(root, helper), join(fixture, helper));
  for (const dependency of [
    "tools/support/compat/github/js-test-cases.ts",
    "tools/support/compat/github/test-case-reporter.ts",
    "tools/support/compat/github/test-cases.ts",
    "tools/support/json-value.ts",
  ]) {
    mkdirSync(dirname(join(fixture, dependency)), { recursive: true });
    copyFileSync(join(root, dependency), join(fixture, dependency));
  }
  writeFileSync(join(fixture, "npm/native/vize-vitrine.fixture.node"), "fixture addon");
}

function commitFixture(fixture) {
  const git = (...args) => execFileSync("git", args, { cwd: fixture, stdio: "ignore" });
  git("init", "-q");
  git("config", "user.name", "CI Fixture");
  git("config", "user.email", "ci@example.invalid");
  git("add", ".");
  git("commit", "-qm", "fixture");
}

void test("prepared reporter sessions survive filtered tasks without inheriting unrelated variables", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-prepared-reporter-'$-"));
  const packet = join(fixture, "test-cases.json");
  try {
    stageNative(fixture);
    writeFileSync(join(fixture, "package.json"), '{"type":"module"}\n');
    writeFileSync(
      join(fixture, "filtered.test.ts"),
      `import assert from 'node:assert/strict'; import {test} from 'node:test';
      test('filtered reporter case',()=>{assert.equal(process.execPath,${JSON.stringify(realpathSync(process.execPath))}); assert.equal(process.env.VIZE_REPORTER_UNKNOWN,undefined);});\n`,
    );
    writeFileSync(
      join(fixture, "filtered.sh"),
      'exec env -i PATH="$PATH" node --test filtered.test.ts\n',
    );
    commitFixture(fixture);
    const run = () =>
      spawnSync(process.execPath, [helper, "sh", "filtered.sh"], {
        cwd: fixture,
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_ACTIONS: "true",
          VIZE_TEST_CASES: packet,
          VIZE_TEST_CASES_ROOT: fixture,
          TEST_CASES_JOB: "prepared native fixture reporter",
          GITHUB_RUN_ATTEMPT: "1",
          NODE_TEST_CONTEXT: undefined,
          VIZE_REPORTER_UNKNOWN: "caller-only",
        },
      });
    const filtered = run();
    assert.equal(filtered.status, 0, `${filtered.stdout}\n${filtered.stderr}`);
    assert.deepEqual(JSON.parse(readFileSync(packet, "utf8")), {
      version: 1,
      job: "prepared native fixture reporter",
      attempt: 1,
      cases: [
        {
          path: "filtered.test.ts",
          name: "filtered.test.ts › filtered reporter case",
          outcome: "passed",
        },
      ],
    });
    rmSync(join(fixture, "tools/support/compat/github/test-case-reporter.ts"));
    const missing = run();
    assert.notEqual(missing.status, 0, `${missing.stdout}\n${missing.stderr}`);
    assert.match(missing.stderr, /test-case-reporter\.ts/u);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("standalone Vite tests build first; prepared tests reuse the same native build", () => {
  const manifest = JSON.parse(readFileSync(join(root, "npm/builder/vite/package.json"), "utf8"));
  assert.equal(
    manifest.scripts["test:prepared"],
    'node scripts/run-tests.mjs --require-prepared && node --test --test-concurrency=1 "src/**/*.test.ts"',
  );
  for (const prepared of [false, true]) {
    const calls = [];
    assert.equal(
      runViteTests(prepared, (command, args, options) => {
        calls.push({ command, args });
        assert.equal(options.env, process.env);
        assert.equal(options.cwd, join(root, "npm/builder/vite"));
        assert.equal(options.stdio, "inherit");
        assert.equal(options.shell, process.platform === "win32");
        return { status: 0 };
      }),
      0,
    );
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

void test("receipts reject source/addon drift and another process identity and clean up on failure", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-native-owner-"));
  const receipt = join(fixture, "npm/native/.artifacts/native/js-test-preparation.json");
  try {
    stageNative(fixture);
    writeFileSync(join(fixture, "package.json"), '{"type":"module"}\n');
    writeFileSync(join(fixture, "tracked-source"), "source\n");
    writeFileSync(
      join(fixture, "probe.mjs"),
      `
import assert from 'node:assert/strict'; import fs from 'node:fs';
import {nativePreparationIsActive,withPreparedNative} from './${helper}';
const dir=${JSON.stringify(join(fixture, "npm/native"))}; const file=${JSON.stringify(receipt)};
const original=JSON.parse(fs.readFileSync(file,'utf8')); const active=()=>nativePreparationIsActive(dir);
assert.equal(active(),true);
for(const field of ['head','tree','workingDiff','sha256','ownerArgument']) {
  fs.writeFileSync(file,JSON.stringify({...original,[field]:'different'})); assert.equal(active(),false);
}
fs.writeFileSync(file,JSON.stringify({...original,ownerPid:process.pid})); assert.equal(active(),false);
fs.writeFileSync(file,JSON.stringify(original));
assert.throws(()=>withPreparedNative(dir,original.ownerArgument,'unused',[]),/owner|already active/);
fs.appendFileSync('tracked-source','changed'); assert.equal(active(),false);
assert.throws(()=>withPreparedNative(dir,original.ownerArgument,'unused',[]),/owner|already active/);
fs.writeFileSync('tracked-source','source\\n'); assert.equal(active(),true);
process.exit(7);
`,
    );
    commitFixture(fixture);
    const run = () =>
      spawnSync(process.execPath, [helper, process.execPath, "probe.mjs"], {
        cwd: fixture,
        encoding: "utf8",
      });
    const first = run();
    assert.equal(first.status, 7, `${first.stdout}\n${first.stderr}`);
    assert.equal(existsSync(receipt), false);
    mkdirSync(dirname(receipt), { recursive: true });
    writeFileSync(receipt, JSON.stringify({ schemaVersion: 1, ownerPid: 2147483647 }));
    const stale = run();
    assert.equal(stale.status, 7, `${stale.stdout}\n${stale.stderr}`);
    assert.equal(existsSync(receipt), false);
    const bin = join(fixture, "bin");
    mkdirSync(bin);
    writeFileSync(join(bin, "ps"), `#!${process.execPath}\nprocess.exit(1);\n`, { mode: 0o755 });
    writeFileSync(
      join(fixture, "fallback.mjs"),
      `import assert from 'node:assert/strict'; import fs from 'node:fs'; assert.equal(fs.existsSync(${JSON.stringify(receipt)}),false); process.exit(4);\n`,
    );
    const fallback = spawnSync(process.execPath, [helper, process.execPath, "fallback.mjs"], {
      cwd: fixture,
      encoding: "utf8",
      env: { ...process.env, PATH: `${bin}:${process.env.PATH}` },
    });
    assert.equal(fallback.status, 4, `${fallback.stdout}\n${fallback.stderr}`);
    assert.match(fallback.stdout, /Native reuse is unavailable/);
    assert.equal(existsSync(receipt), false);
    writeFileSync(
      join(fixture, "windows.mjs"),
      `
import assert from 'node:assert/strict'; import fs from 'node:fs';
import {withPreparedNative} from './${helper}';
import {runViteTests} from ${JSON.stringify(join(root, "npm/builder/vite/scripts/run-tests.mjs"))};
Object.defineProperty(process,'platform',{value:'win32'});
let calls=0;
const packageRun=(command,args,options)=>{
  if(command===process.execPath) {
    assert.ok(process.env.VIZE_TEST_CASES);
    assert.equal(fs.realpathSync(args[0]),fs.realpathSync(${JSON.stringify(join(fixture, "tools/support/compat/github/js-test-cases.ts"))}));
    assert.deepEqual(args.slice(1),['capture','--','vp','run','test']);
    assert.equal(options.shell,false);
  } else { assert.ok(command==='vp'||command==='pnpm'); assert.equal(options.shell,true); }
  assert.equal(options.env,process.env); calls++; return {status:0};
};
assert.equal(withPreparedNative(${JSON.stringify(join(fixture, "npm/native"))},'--native-test-owner=00000000-0000-0000-0000-000000000000','vp',['run','test'],packageRun),0);
assert.equal(runViteTests(false,packageRun),0); assert.equal(calls,3);
`,
    );
    const windows = spawnSync(process.execPath, ["windows.mjs"], {
      cwd: fixture,
      encoding: "utf8",
    });
    assert.equal(windows.status, 0, `${windows.stdout}\n${windows.stderr}`);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("cached root JS tasks preserve selection/order/environment and require successful preparation", () => {
  const before = `${runTask("build:native:test")} && ${runInPackages("test", testedPackages, { concurrencyLimit: 1 })}`;
  const after = testAndBenchmarkTasks["test:js"].command;
  assert.equal(after.replace(`node ${helper} vp run`, "vp run"), before);
  const fixture = mkdtempSync(join(tmpdir(), "vize-prepared-root-"));
  const events = join(fixture, "events.jsonl");
  const vp = process.env.VIZE_VP_BIN ?? join(root, "node_modules/.bin/vp");
  try {
    stageNative(fixture);
    writeFileSync(
      join(fixture, "package.json"),
      JSON.stringify({ name: "prepared-root", type: "module" }),
    );
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
      writeFileSync(join(pkgDir, "vite.config.mjs"), "export default {};\n");
      const imports =
        pkg === "./npm/builder/vite"
          ? `import {nativePreparationIsActive} from '../../native/scripts/test-preparation.mjs';`
          : "";
      const prepared =
        pkg === "./npm/builder/vite"
          ? `nativePreparationIsActive(${JSON.stringify(join(fixture, "npm/native"))})`
          : "null";
      writeFileSync(
        join(pkgDir, "probe.mjs"),
        `${imports} import fs from 'node:fs'; fs.appendFileSync(${JSON.stringify(events)}, JSON.stringify({package:${JSON.stringify(pkg)},prepared:${prepared},env:process.env.VIZE_PREPARED_OTHER})+'\\n');\n`,
      );
    }
    commitFixture(fixture);
    let invocation = 0;
    const run = (command, preparationExit = 0) => {
      invocation += 1;
      rmSync(events, { force: true });
      for (const pkg of testedPackages)
        appendFileSync(join(fixture, pkg, "probe.mjs"), `// invocation ${invocation}\n`);
      writeFileSync(
        join(fixture, "vite.config.mjs"),
        `export default { run: { cache:{scripts:true,tasks:true}, tasks: { 'build:native:test': { command:'node prepare.mjs',cache:false },'test:js':{command:${JSON.stringify(command)},cache:false} } } };\n`,
      );
      const result = spawnSync(vp, ["run", "--workspace-root", "test:js"], {
        cwd: fixture,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${dirname(vp)}:${process.env.PATH}`,
          VIZE_PREPARE_EXIT: String(preparationExit),
          VIZE_PREPARED_OTHER: "same-environment",
        },
      });
      if (preparationExit === 0)
        assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      return { result, records: readFileSync(events, "utf8").trim().split("\n").map(JSON.parse) };
    };
    const baseline = run(before);
    const prepared = run(after);
    for (const { records } of [baseline, prepared]) {
      assert.deepEqual(records[0], { prepare: true });
      assert.equal(records.length, testedPackages.length + 1);
    }
    assert.deepEqual(
      prepared.records.slice(1),
      baseline.records
        .slice(1)
        .map((record) =>
          record.package === "./npm/builder/vite" ? { ...record, prepared: true } : record,
        ),
    );
    assert.equal(
      existsSync(join(fixture, "npm/native/.artifacts/native/js-test-preparation.json")),
      false,
    );
    const failed = run(after, 7);
    assert.notEqual(failed.result.status, 0);
    assert.deepEqual(failed.records, [{ prepare: true }]);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
