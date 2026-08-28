import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const nativePackageDir = path.join(root, "npm/native");
const packageVersion = "1.0.0";

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

function isolatedNativeLoaderEnv(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  const env = { ...process.env };
  for (const key of Object.keys(env)) {
    const normalized = key.toLowerCase();
    if (normalized === "init_cwd" || normalized === "node_path" || normalized.startsWith("npm_")) {
      Reflect.deleteProperty(env, key);
    }
  }
  Reflect.deleteProperty(env, "NAPI_RS_FORCE_WASI");
  Reflect.deleteProperty(env, "NAPI_RS_NATIVE_LIBRARY_PATH");
  Reflect.deleteProperty(env, "VIZE_ALLOW_NATIVE_VERSION_MISMATCH");
  return { ...env, ...extra };
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
  const npmEnv = isolatedNativeLoaderEnv({ npm_config_cache: npmCacheDir });
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

function installBrokenTargetPackage(installDir: string): void {
  const targetPackageName = currentTargetPackageName();
  const targetPackage = path.join(installDir, "node_modules", ...targetPackageName.split("/"));
  fs.mkdirSync(targetPackage, { recursive: true });
  fs.writeFileSync(
    path.join(targetPackage, "package.json"),
    `${JSON.stringify(
      {
        main: "index.js",
        name: targetPackageName,
        version: packageVersion,
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    path.join(targetPackage, "index.js"),
    [
      'const error = new Error("/lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39\' not found");',
      'error.code = "ERR_DLOPEN_FAILED";',
      "throw error;",
      "",
    ].join("\n"),
  );
}

function runProbe(installDir: string, filename: string, source: string) {
  fs.writeFileSync(path.join(installDir, filename), source);
  return run(process.execPath, [filename], { cwd: installDir, env: isolatedNativeLoaderEnv() });
}

test("packed package root distinguishes missing and incompatible native targets", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-error-"));

  try {
    const missingInstallDir = installPackedNativePackage(path.join(tempDir, "missing"));
    const missing = runProbe(
      missingInstallDir,
      "reject-missing.cjs",
      `
try {
  require("@vizejs/native");
  throw new Error("missing binding unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("optional dependencies")) {
    console.error(error.message);
    process.exit(1);
  }
  console.log("package root reported missing optional dependency");
}
`,
    );
    assertSucceeded(missing, "require('@vizejs/native') without target package");
    assert.match(missing.stdout, /reported missing optional dependency/);

    const brokenInstallDir = installPackedNativePackage(path.join(tempDir, "broken"));
    installBrokenTargetPackage(brokenInstallDir);
    const broken = runProbe(
      brokenInstallDir,
      "reject-broken.cjs",
      `
try {
  require("@vizejs/native");
  throw new Error("broken binding unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("GLIBC_2.39")) {
    console.error(error.message);
    process.exit(1);
  }
  if (error.message.includes("optional dependencies")) {
    console.error(error.message);
    process.exit(1);
  }
  console.log("package root surfaced native loader failure");
}
`,
    );
    assertSucceeded(broken, "require('@vizejs/native') with incompatible target package");
    assert.match(broken.stdout, /surfaced native loader failure/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
