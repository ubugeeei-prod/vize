import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const nativePackageDir = path.join(root, "npm/native");
const entrypointSyncScript = path.join(nativePackageDir, "scripts/sync-entrypoint.mjs");
const packageVersion = "1.0.0";
const staleTargetVersion = "0.9.0";

function run(
  command: string,
  args: string[],
  options: { cwd: string; env?: NodeJS.ProcessEnv },
): { status: number | null; stdout: string; stderr: string } {
  const isWindowsBatch = process.platform === "win32" && /\.(cmd|bat)$/i.test(command);
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: "utf8",
    env: options.env ?? process.env,
    shell: isWindowsBatch,
  });

  if (result.error != null) {
    throw result.error;
  }

  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

function assertSucceeded(
  result: { status: number | null; stdout: string; stderr: string },
  command: string,
): void {
  assert.equal(result.status, 0, `${command}\n${result.stderr}\n${result.stdout}`.trim());
}

function currentTargetPackageName(): string {
  if (process.platform === "darwin") {
    return "@vizejs/native-darwin-universal";
  }
  if (process.platform === "win32" && (process.arch === "x64" || process.arch === "arm64")) {
    return `@vizejs/native-win32-${process.arch}-msvc`;
  }
  if (process.platform === "linux" && (process.arch === "x64" || process.arch === "arm64")) {
    const report = process.report?.getReport() as
      | { header?: { glibcVersionRuntime?: unknown } }
      | undefined;
    const isGlibc = typeof report?.header?.glibcVersionRuntime === "string";
    return `@vizejs/native-linux-${process.arch}-${isGlibc ? "gnu" : "musl"}`;
  }
  throw new Error(`Unsupported native loader test host: ${process.platform}-${process.arch}`);
}

function copyPackableNativePackage(destination: string): void {
  fs.mkdirSync(destination, { recursive: true });
  for (const file of [
    "index.js",
    "index.d.ts",
    "native-binding.js",
    "native-targets.js",
    "README.md",
  ]) {
    fs.copyFileSync(path.join(nativePackageDir, file), path.join(destination, file));
  }
  const manifest = JSON.parse(
    fs.readFileSync(path.join(nativePackageDir, "package.json"), "utf8"),
  ) as Record<string, unknown>;
  manifest.version = packageVersion;
  manifest.optionalDependencies = {};
  manifest.devDependencies = {};
  manifest.scripts = {};
  fs.writeFileSync(
    path.join(destination, "package.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
}

function installPackedNativePackage(tempDir: string): string {
  const packageSourceDir = path.join(tempDir, "package-source");
  const packDir = path.join(tempDir, "packed");
  const installDir = path.join(tempDir, "install");
  const npmCacheDir = path.join(tempDir, "npm-cache");
  fs.mkdirSync(packDir, { recursive: true });
  fs.mkdirSync(installDir, { recursive: true });
  copyPackableNativePackage(packageSourceDir);

  const npm = process.env.NPM_BIN ?? (process.platform === "win32" ? "npm.cmd" : "npm");
  const npmEnv = { ...process.env, npm_config_cache: npmCacheDir };
  const pack = run(npm, ["pack", "--ignore-scripts", "--pack-destination", packDir], {
    cwd: packageSourceDir,
    env: npmEnv,
  });
  assertSucceeded(pack, "npm pack");
  const tarballs = fs.readdirSync(packDir).filter((entry) => entry.endsWith(".tgz"));
  assert.equal(
    tarballs.length,
    1,
    `expected one native package tarball, got ${tarballs.join(", ")}`,
  );

  fs.writeFileSync(
    path.join(installDir, "package.json"),
    `${JSON.stringify({ name: "native-loader-smoke", private: true, version: "1.0.0" }, null, 2)}\n`,
  );
  const tarballPath = path.join(packDir, tarballs[0]);
  const install = run(
    npm,
    [
      "install",
      "--ignore-scripts",
      "--no-save",
      "--omit=optional",
      "--no-audit",
      "--no-fund",
      tarballPath,
    ],
    { cwd: installDir, env: npmEnv },
  );
  assertSucceeded(install, "npm install <packed @vizejs/native>");

  return installDir;
}

function installFakeTargetPackage(installDir: string): void {
  const targetPackageName = currentTargetPackageName();
  const targetPackage = path.join(installDir, "node_modules", ...targetPackageName.split("/"));
  fs.mkdirSync(targetPackage, { recursive: true });
  fs.writeFileSync(
    path.join(targetPackage, "package.json"),
    `${JSON.stringify(
      {
        main: "index.js",
        name: targetPackageName,
        version: staleTargetVersion,
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    path.join(targetPackage, "index.js"),
    [
      'function compileSfc() { return "compiled by stale binding"; }',
      'module.exports = { compileSfc, marker: "loaded stale binding" };',
      "",
    ].join("\n"),
  );
}

function runProbe(
  installDir: string,
  filename: string,
  source: string,
  env: NodeJS.ProcessEnv = process.env,
): { status: number | null; stdout: string; stderr: string } {
  fs.writeFileSync(path.join(installDir, filename), source);
  return run(process.execPath, [filename], { cwd: installDir, env });
}

test("generated native entrypoint delegates loading without losing static named exports", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-entrypoint-sync-"));
  const generatedEntrypoint = `
const loadErrors = [];
function requireNative() { return null; }
let nativeBinding = requireNative();
module.exports = nativeBinding
module.exports.compile = nativeBinding.compile
module.exports.compileSfc = nativeBinding.compileSfc
`;

  try {
    const generatedPath = path.join(tempDir, "generated-index.js");
    fs.writeFileSync(generatedPath, generatedEntrypoint);
    const generatedSync = run(process.execPath, [entrypointSyncScript, generatedPath], {
      cwd: root,
    });
    assertSucceeded(generatedSync, "sync generated NAPI entrypoint");

    const synchronized = fs.readFileSync(generatedPath, "utf8");
    assert.match(synchronized, /const nativeBinding = require\("\.\/native-binding"\);/);
    assert.doesNotMatch(synchronized, /requireNative|loadErrors/);
    assert.match(synchronized, /module\.exports\.compile = nativeBinding\.compile;/);
    assert.match(synchronized, /module\.exports\.compileSfc = nativeBinding\.compileSfc;/);

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
  assert.match(
    manifest.scripts?.["build:ci"] ?? "",
    /napi build[\s\S]*&& node \.\/scripts\/sync-entrypoint\.mjs$/,
  );
  assert.match(
    manifest.scripts?.prepublishOnly ?? "",
    /napi pre-publish[\s\S]*&& node \.\/scripts\/sync-entrypoint\.mjs$/,
  );
});

test("packed package root rejects stale bindings by default and keeps the explicit escape hatch", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-package-"));

  try {
    const installDir = installPackedNativePackage(tempDir);
    installFakeTargetPackage(installDir);

    const rejected = runProbe(
      installDir,
      "reject-stale.cjs",
      `
function causeMessages(error) {
  const messages = [];
  let current = error;
  while (current) {
    messages.push(current.message);
    current = current.cause;
  }
  return messages.join("\\n");
}

try {
  require("@vizejs/native");
  throw new Error("stale binding unexpectedly loaded");
} catch (error) {
  const messages = causeMessages(error);
  if (
    !messages.includes("Native binding package version mismatch") ||
    !messages.includes("expected ${packageVersion} but got ${staleTargetVersion}")
  ) {
    console.error(messages);
    process.exit(1);
  }
  console.log("package root rejected stale binding");
}
`,
    );
    assertSucceeded(rejected, "require('@vizejs/native') without escape hatch");
    assert.match(rejected.stdout, /package root rejected stale binding/);

    const allowed = runProbe(
      installDir,
      "allow-stale.cjs",
      `
(async () => {
  const cjs = require("@vizejs/native");
  const esm = await import("@vizejs/native");
  if (cjs.marker !== "loaded stale binding") throw new Error("CJS package root did not load");
  if (cjs.compileSfc() !== "compiled by stale binding") throw new Error("CJS export missing");
  if (esm.default !== cjs) throw new Error("ESM default export does not match CJS");
  if (esm.compileSfc !== cjs.compileSfc) throw new Error("ESM named export missing");
  console.log("package root escape hatch preserved CJS and ESM exports");
})().catch((error) => {
  console.error(error);
  process.exit(1);
});
`,
      { ...process.env, VIZE_ALLOW_NATIVE_VERSION_MISMATCH: "1" },
    );
    assertSucceeded(allowed, "require/import('@vizejs/native') with escape hatch");
    assert.match(allowed.stdout, /escape hatch preserved CJS and ESM exports/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
