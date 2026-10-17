import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  isPathInsideOrEqual,
  runInitTypecheckChecks,
} from "../../../tools/support/compat/npm/smoke-release-init-typecheck.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const archive = path.join(
  root,
  "tests/_fixtures/differential/config/project-settings-8371/archives",
);

function invertTypescriptDriverSuccessor(current: string, original: string): string {
  const replaceOnce = (successor: string, predecessor: string) => {
    assert.equal(current.split(successor).length, 2, "undeclared runtime oracle successor");
    current = current.replace(successor, predecessor);
  };
  replaceOnce(
    [
      'const typescriptDriver = process.env.VIZE_RELEASE_SMOKE_DRIVER === "typescript";',
      'const command = typescriptDriver ? process.execPath : "rust-script";',
      'const entry = `tools/commands/release/npm/smoke-release-install.${typescriptDriver ? "ts" : "rs"}`;',
      'const prefix = typescriptDriver ? [] : ["--force"];',
      "",
    ].join("\n"),
    "",
  );
  replaceOnce(
    'test("the release command forwards all tarballs to the fresh-project oracle",',
    'test("the Rust release command forwards all tarballs to the fresh-project oracle",',
  );
  for (const kind of ["dispatch", "oracle"]) {
    replaceOnce(
      `  const temporary = fs.realpathSync.native(\n    fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-${kind}-")),\n  );`,
      `  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-${kind}-"));`,
    );
  }
  replaceOnce(
    "assert.equal(context.repoRoot, fs.realpathSync.native(process.env.VIZE_REPO_ROOT));",
    "assert.equal(context.repoRoot, fs.realpathSync(process.env.VIZE_REPO_ROOT));",
  );
  // These complete successor calls retain every original argument, environment
  // and refusal check. Only the reviewed driver selector and Windows canonical
  // roots change. An altered option cannot disappear behind the inverse.
  const calls = [
    `    const result = spawnSync(command, [...prefix, entry, "--runtime-checks", cli, platform], {
      cwd: root,
      encoding: "utf8",
      timeout: 180_000,
      env: {
        ...process.env,
        VIZE_REPO_ROOT: fs.realpathSync.native(temporary),
        COREPACK_HOME: path.join(temporary, "stale-host-corepack"),
        CORSA_PATH: "/unrelated/host/corsa",
        CORSA_EXECUTABLE: "/unrelated/host/corsa",
        TSGO_PATH: "/unrelated/host/tsgo",
        TSGO_EXECUTABLE: "/unrelated/host/tsgo",
      },
    });
`,
    `    const result = spawnSync(command, [...prefix, entry, "--runtime-checks", packageRoot], {
      cwd: root,
      encoding: "utf8",
      timeout: 180_000,
    });
`,
  ];
  const originalCalls = [
    ...original.matchAll(
      /    const result = spawnSync\(\n[\s\S]+?(?=    assert\.ifError\(result\.error\);)/gu,
    ),
  ];
  assert.equal(originalCalls.length, 2);
  for (const [index, call] of calls.entries()) replaceOnce(call, originalCalls[index][0]);
  return current;
}

function installedPreludeFixture(t: TestContext, mode: string) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-init-file-policy-"));
  const installDir = path.join(temporary, "install");
  const vizeBin = path.join(installDir, "node_modules/vize/bin/vize");
  fs.mkdirSync(path.dirname(vizeBin), { recursive: true });
  fs.writeFileSync(
    vizeBin,
    `const fs = require("node:fs");
const path = require("node:path");
const project = process.argv[3];
const mode = ${JSON.stringify(mode)};
const counter = path.join(project, ".fixture-init-count");
const count = fs.existsSync(counter) ? Number(fs.readFileSync(counter, "utf8")) + 1 : 1;
fs.writeFileSync(counter, String(count));
const filename = path.join(project, "package.json");
const manifest = JSON.parse(fs.readFileSync(filename, "utf8"));
manifest.scripts = { "vize:check": "vize check" };
if (mode === "second-package" && count === 2) manifest.changedOnRerun = true;
fs.writeFileSync(filename, JSON.stringify(manifest));
const js = !manifest.devDependencies.typescript;
fs.writeFileSync(path.join(project, "tsconfig.json"), JSON.stringify({
  compilerOptions: js ? { allowJs: true, checkJs: true } : {},
  ...(mode === "second-tsconfig" && count === 2 ? { changedOnRerun: true } : {}),
}));
if ((mode === "first-config" && count === 1) || (mode === "second-config" && count === 2)) {
  fs.writeFileSync(path.join(project, "vize.config.mjs"), "export default {};\\n");
}
fs.writeFileSync(path.join(project, "run"), 'console.log(JSON.stringify({ errorCount: 0, files: [] }));');
`,
  );
  const previousNpm = process.env.NPM_BIN;
  process.env.NPM_BIN = process.execPath;
  t.after(() => {
    if (previousNpm === undefined) delete process.env.NPM_BIN;
    else process.env.NPM_BIN = previousNpm;
    fs.rmSync(temporary, { recursive: true, force: true });
  });
  return { installDir, vizeBin, temporary };
}

const peers = { typescript: "6.0.3", vue: "3.5.34" };

test("init typecheck smoke rejects only package roots inside the repository", () => {
  assert.equal(
    isPathInsideOrEqual("D:\\a\\vize\\vize", "D:\\a\\vize\\vize\\node_modules\\vize", path.win32),
    true,
  );
  assert.equal(
    isPathInsideOrEqual("D:\\a\\vize\\vize", "D:\\a\\vize\\other\\node_modules\\vize", path.win32),
    false,
  );
  assert.equal(
    isPathInsideOrEqual(
      "D:\\a\\vize\\vize",
      "C:\\Users\\runneradmin\\AppData\\Local\\Temp\\vize-release-smoke-AAk6yG\\install\\node_modules\\vize",
      path.win32,
    ),
    false,
  );
  assert.equal(isPathInsideOrEqual("/tmp/vize", "/tmp/vize/node_modules/vize", path.posix), true);
  assert.equal(
    isPathInsideOrEqual("/tmp/vize", "/tmp/vize-other/node_modules/vize", path.posix),
    false,
  );
});

test("init typecheck smoke treats .. as a path segment", () => {
  assert.equal(
    isPathInsideOrEqual(
      "D:\\a\\vize\\vize",
      "D:\\a\\vize\\vize\\..cache\\node_modules\\vize",
      path.win32,
    ),
    true,
  );
  assert.equal(isPathInsideOrEqual("D:\\a\\vize\\vize", "D:\\a\\vize", path.win32), false);
  assert.equal(
    isPathInsideOrEqual("/tmp/vize", "/tmp/vize/..cache/node_modules/vize", path.posix),
    true,
  );
  assert.equal(isPathInsideOrEqual("/tmp/vize", "/tmp", path.posix), false);
});

test("installed init rejects a dedicated config from either initialization pass", async (t) => {
  for (const mode of ["first-config", "second-config"]) {
    await t.test(mode, (t) => {
      const fixture = installedPreludeFixture(t, mode);
      assert.throws(
        () => runInitTypecheckChecks(fixture.installDir, fixture.vizeBin, root, peers),
        /the current initializer generated a dedicated Vize config/u,
      );
    });
  }
});

test("installed init retains both complete generated snapshots on its second pass", async (t) => {
  for (const mode of ["second-package", "second-tsconfig"]) {
    await t.test(mode, (t) => {
      const fixture = installedPreludeFixture(t, mode);
      assert.throws(
        () => runInitTypecheckChecks(fixture.installDir, fixture.vizeBin, root, peers),
        /typescript init should be idempotent/u,
      );
    });
  }
});

test("config-free init reaches the unchanged false-clean diagnostic control", async (t) => {
  const fixture = installedPreludeFixture(t, "config-free");
  assert.throws(
    () => runInitTypecheckChecks(fixture.installDir, fixture.vizeBin, root, peers),
    /typescript standalone source unexpectedly passed/u,
  );
  const original = fs.readFileSync(
    path.join(archive, "smoke-release-init-typecheck.original.mjs.txt"),
    "utf8",
  );
  const originalPath = path.join(fixture.temporary, "original-helper.mjs");
  fs.writeFileSync(originalPath, original);
  const archived = await import(pathToFileURL(originalPath).href);
  assert.throws(
    () => archived.runInitTypecheckChecks(fixture.installDir, fixture.vizeBin, root, peers),
    /ENOENT.*vize\.config\.ts/u,
  );
});

test("the declared generated-file successor restores both entire archival sources", () => {
  const originalHelper = fs.readFileSync(
    path.join(archive, "smoke-release-init-typecheck.original.mjs.txt"),
    "utf8",
  );
  const currentHelper = fs.readFileSync(
    path.join(root, "tools/support/compat/npm/smoke-release-init-typecheck.mjs"),
    "utf8",
  );
  assert.equal(
    createHash("sha256").update(originalHelper).digest("hex"),
    "2ce7b4aa4ffde080bc916cd42edcfd507010ca7d6eddef65d8f951b642d5c107",
  );
  assert.equal(currentHelper.match(/assertNoDedicatedConfig\(projectDir\);/gu)?.length, 2);
  const inverseHelper = currentHelper
    .replace('import { assertNoDedicatedConfig } from "./smoke-release-init-fresh.mjs";\n', "")
    .replaceAll("  assertNoDedicatedConfig(projectDir);\n", "")
    .replace(
      '  const tsconfigPath = path.join(projectDir, "tsconfig.json");\n',
      '  const tsconfigPath = path.join(projectDir, "tsconfig.json");\n' +
        '  const configPath = path.join(projectDir, "vize.config.ts");\n',
    )
    .replaceAll("[packagePath, tsconfigPath]", "[packagePath, tsconfigPath, configPath]");
  assert.equal(inverseHelper, originalHelper);

  const originalOracle = fs.readFileSync(
    path.join(archive, "release-smoke-runtime-oracle.original.test.ts.txt"),
    "utf8",
  );
  const currentOracle = invertTypescriptDriverSuccessor(
    fs.readFileSync(
      path.join(root, "tests/tooling/release/release-smoke-runtime-oracle.test.ts"),
      "utf8",
    ),
    originalOracle,
  );
  assert.equal(
    createHash("sha256").update(originalOracle).digest("hex"),
    "ad5ef9636ea6310104c1dd6ecd43cd11b7a09ab8178a29145cd0b390d5edc8c0",
  );
  const originalStart = originalOracle.indexOf(
    '    fs.writeFileSync(\n      path.join(packageRoot, "bin/vize"),\n',
  );
  const currentStart = currentOracle.indexOf("    const historicalFalseCleanCli = ");
  const end = "    const result = spawnSync(\n";
  const originalEnd = originalOracle.indexOf(end, originalStart);
  const currentEnd = currentOracle.indexOf(end, currentStart);
  assert.ok(
    originalStart >= 0 &&
      currentStart >= 0 &&
      originalEnd > originalStart &&
      currentEnd > currentStart,
  );
  const originalLiteral = originalOracle.slice(
    originalOracle.indexOf("`#!/usr/bin/env node", originalStart),
    originalOracle.indexOf("`,\n", originalStart) + 1,
  );
  assert.ok(currentOracle.includes(`const historicalFalseCleanCli = ${originalLiteral};`));
  assert.ok(
    currentOracle.includes("const configFreeFalseCleanCli = historicalFalseCleanCli.replace("),
  );
  assert.equal(
    currentOracle.slice(currentStart, currentEnd),
    `    const historicalFalseCleanCli = ${originalLiteral};
    const configFreeFalseCleanCli = historicalFalseCleanCli.replace(
      '  fs.writeFileSync(path.join(project, "vize.config.ts"), "export default {};\\\\n");\\n',
      "",
    );
    fs.writeFileSync(path.join(packageRoot, "bin/vize"), configFreeFalseCleanCli, { mode: 0o755 });
`,
  );
  const inverseOracle =
    currentOracle.slice(0, currentStart) +
    originalOracle.slice(originalStart, originalEnd) +
    currentOracle.slice(currentEnd);
  assert.equal(inverseOracle, originalOracle);
});

test("the driver successor inverse refuses undeclared dispatcher or invocation changes", () => {
  const original = fs.readFileSync(
    path.join(archive, "release-smoke-runtime-oracle.original.test.ts.txt"),
    "utf8",
  );
  const current = fs.readFileSync(
    path.join(root, "tests/tooling/release/release-smoke-runtime-oracle.test.ts"),
    "utf8",
  );
  for (const [before, after] of [
    [
      'const command = typescriptDriver ? process.execPath : "rust-script";',
      'const command = "npm";',
    ],
    ['[...prefix, entry, "--runtime-checks", cli, platform]', "[...prefix, entry, cli]"],
    ["timeout: 180_000", "timeout: 360_000"],
    ['CORSA_PATH: "/unrelated/host/corsa"', "CORSA_PATH: process.env.CORSA_PATH"],
  ]) {
    assert.notEqual(current.indexOf(before), -1);
    assert.throws(
      () => invertTypescriptDriverSuccessor(current.replace(before, after), original),
      /undeclared runtime oracle successor/u,
    );
  }
});
