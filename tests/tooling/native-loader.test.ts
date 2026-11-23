import assert from "node:assert/strict";
import {
  assertSucceeded,
  installNativeTarget,
  installPackedNativePackage,
  nativePackageDir,
  packageVersion,
  run,
  runNativeLoaderProbe,
} from "./support/native-loader-package.ts";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { toolingProgram } from "./support/program-path.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const entrypointSyncScript = path.join(nativePackageDir, "scripts/sync-entrypoint.mjs");
const staleTargetVersion = "0.9.0";

test("generated native entrypoint delegates loading without losing static named exports", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-entrypoint-sync-"));

  try {
    const generatedPath = path.join(tempDir, "generated-index.js");
    fs.copyFileSync(toolingProgram("native/unsynchronized-entrypoint.js.in"), generatedPath);
    const generatedSync = run(process.execPath, [entrypointSyncScript, generatedPath], {
      cwd: root,
    });
    assertSucceeded(generatedSync, "sync generated NAPI entrypoint");

    const synchronized = fs.readFileSync(generatedPath, "utf8");
    assert.match(synchronized, /const nativeBinding = require\("\.\/native-binding"\);/);
    assert.doesNotMatch(synchronized, /requireNative|loadErrors/);
    assert.match(synchronized, /module\.exports\.compile = nativeBinding\.compile;/);
    assert.match(synchronized, /module\.exports\.compileSfc = nativeBinding\.compileSfc;/);
    assert.match(synchronized, /const compileSfcNative = nativeBinding\.compileSfc;/);
    assert.match(synchronized, /queueMicrotask/);

    const idempotentSync = run(process.execPath, [entrypointSyncScript, generatedPath], {
      cwd: root,
    });
    assertSucceeded(idempotentSync, "sync synchronized NAPI entrypoint");
    assert.equal(fs.readFileSync(generatedPath, "utf8"), synchronized, "sync must be idempotent");

    const invalidFooters = [
      ["wrapped", "module.exports.compileSfc = wrappedCompileSfc;", /Unsupported generated/],
      ["remapped", "module.exports.compileSfc = nativeBinding.compile;", /remapped native export/],
      [
        "duplicate",
        [
          "module.exports.compile = nativeBinding.compile;",
          "module.exports.compile = nativeBinding.compile;",
        ].join("\n"),
        /Duplicate generated native export/,
      ],
    ] as const;
    for (const [name, footer, expectedError] of invalidFooters) {
      const unsupportedPath = path.join(tempDir, `${name}-index.js`);
      fs.writeFileSync(
        unsupportedPath,
        `const nativeBinding = requireNative();\nmodule.exports = nativeBinding;\n${footer}\n`,
      );
      const unsupportedSync = run(process.execPath, [entrypointSyncScript, unsupportedPath], {
        cwd: root,
      });
      assert.notEqual(unsupportedSync.status, 0, `${name} footer must be rejected`);
      assert.match(unsupportedSync.stderr, expectedError);
    }
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }

  const committedEntrypoint = fs.readFileSync(path.join(nativePackageDir, "index.js"), "utf8");
  assert.match(committedEntrypoint, /const nativeBinding = require\("\.\/native-binding"\);/);
  assert.doesNotMatch(committedEntrypoint, /NAPI_RS_ENFORCE_VERSION_CHECK|0\.272\.0/);

  const check = run(process.execPath, [entrypointSyncScript, "--check"], {
    cwd: root,
  });
  assertSucceeded(check, "node npm/native/scripts/sync-entrypoint.mjs --check");

  const manifest = JSON.parse(
    fs.readFileSync(path.join(nativePackageDir, "package.json"), "utf8"),
  ) as { scripts?: Record<string, string> };
  assert.equal(manifest.scripts?.["build:ci"], "node ./scripts/build-ci.mjs");
  assert.equal(manifest.scripts?.prepublishOnly, "node ./scripts/package-tasks.cjs prepublishOnly");
  const { tasks } = createRequire(import.meta.url)(
    path.join(nativePackageDir, "scripts/package-tasks.cjs"),
  );
  assert.deepEqual(tasks.prepublishOnly, [
    {
      program: "napi",
      arguments: ["pre-publish", "-t", "npm", "--no-gh-release", "--skip-optional-publish"],
    },
    { program: "node", arguments: ["./scripts/sync-entrypoint.mjs"] },
  ]);
});

test("packed package root rejects stale bindings by default and keeps the explicit escape hatch", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-package-"));

  try {
    const installDir = installPackedNativePackage(tempDir);
    installNativeTarget(installDir, "stale-binding.cjs", staleTargetVersion);

    const rejected = runNativeLoaderProbe(installDir, "reject-stale.cjs", [
      packageVersion,
      staleTargetVersion,
    ]);
    assertSucceeded(rejected, "require('@vizejs/native') without escape hatch");
    assert.match(rejected.stdout, /package root rejected stale binding/);

    const allowed = runNativeLoaderProbe(installDir, "allow-stale.cjs", [], {
      ...process.env,
      VIZE_ALLOW_NATIVE_VERSION_MISMATCH: "1",
    });
    assertSucceeded(allowed, "require/import('@vizejs/native') with escape hatch");
    assert.match(allowed.stdout, /escape hatch preserved CJS and ESM exports/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("generated native entrypoint shares imported-type validation for one JavaScript turn", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-entrypoint-turn-"));
  try {
    const entrypoint = path.join(tempDir, "index.js");
    fs.copyFileSync(toolingProgram("native/turn-entrypoint.js.in"), entrypoint);
    fs.copyFileSync(
      toolingProgram("native/turn-binding.cjs"),
      path.join(tempDir, "native-binding.js"),
    );
    const synchronized = run(process.execPath, [entrypointSyncScript, entrypoint], { cwd: root });
    assertSucceeded(synchronized, "sync turn-scoped NAPI entrypoint");
    const probe = run(
      process.execPath,
      [
        "--input-type=commonjs",
        toolingProgram("native/count-native-validation-turns.cjs"),
        JSON.stringify({ nativeLibrary: entrypoint }),
      ],
      { cwd: root },
    );
    assertSucceeded(probe, "probe turn-scoped compileSfc wrapper");
    assert.deepEqual(JSON.parse(probe.stdout), {
      first: { begins: 1, closes: 1 },
      second: { begins: 2, closes: 2 },
    });
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
