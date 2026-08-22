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
  cwd: string,
): { status: number | null; stdout: string; stderr: string } {
  const result = spawnSync(command, args, { cwd, encoding: "utf8", env: process.env });
  if (result.error != null) throw result.error;
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

function assertSucceeded(
  result: { status: number | null; stdout: string; stderr: string },
  command: string,
): void {
  assert.equal(result.status, 0, `${command}\n${result.stderr}\n${result.stdout}`.trim());
}

function currentTargetPackageName(): string {
  if (process.platform === "darwin") return "@vizejs/native-darwin-universal";
  if (process.platform === "win32" && (process.arch === "x64" || process.arch === "arm64")) {
    return `@vizejs/native-win32-${process.arch}-msvc`;
  }
  if (process.platform === "linux" && (process.arch === "x64" || process.arch === "arm64")) {
    const report = process.report?.getReport() as
      | { header?: { glibcVersionRuntime?: unknown } }
      | undefined;
    return `@vizejs/native-linux-${process.arch}-${typeof report?.header?.glibcVersionRuntime === "string" ? "gnu" : "musl"}`;
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
  const pack = spawnSync(npm, ["pack", "--ignore-scripts", "--pack-destination", packDir], {
    cwd: packageSourceDir,
    encoding: "utf8",
    env: npmEnv,
  });
  assertSucceeded(pack, "npm pack");
  const tarballs = fs.readdirSync(packDir).filter((entry) => entry.endsWith(".tgz"));
  assert.equal(tarballs.length, 1);
  fs.writeFileSync(
    path.join(installDir, "package.json"),
    '{"name":"native-loader-failures","private":true}\n',
  );
  const install = spawnSync(
    npm,
    [
      "install",
      "--ignore-scripts",
      "--no-save",
      "--omit=optional",
      "--no-audit",
      "--no-fund",
      path.join(packDir, tarballs[0]),
    ],
    { cwd: installDir, encoding: "utf8", env: npmEnv },
  );
  assertSucceeded(install, "npm install <packed @vizejs/native>");
  return installDir;
}

function installFakeTargetPackage(installDir: string, source: string): void {
  const packageName = currentTargetPackageName();
  const targetPackage = path.join(installDir, "node_modules", ...packageName.split("/"));
  fs.mkdirSync(targetPackage, { recursive: true });
  fs.writeFileSync(
    path.join(targetPackage, "package.json"),
    `${JSON.stringify({ main: "index.js", name: packageName, version: packageVersion }, null, 2)}\n`,
  );
  fs.writeFileSync(path.join(targetPackage, "index.js"), source);
}

function runProbe(
  installDir: string,
  source: string,
): { status: number | null; stdout: string; stderr: string } {
  fs.writeFileSync(path.join(installDir, "probe.cjs"), source);
  return run(process.execPath, ["probe.cjs"], installDir);
}

test("packed package root surfaces dlopen failures", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-dlopen-"));
  try {
    const installDir = installPackedNativePackage(tempDir);
    installFakeTargetPackage(
      installDir,
      [
        'const error = new Error("/lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39` not found");',
        'error.code = "ERR_DLOPEN_FAILED";',
        "throw error;",
        "",
      ].join("\n"),
    );
    const rejected = runProbe(
      installDir,
      `
function hasCauseCode(error, code) {
  let current = error;
  while (current) {
    if (current.code === code) return true;
    current = current.cause;
  }
  return false;
}

try {
  require("@vizejs/native");
  throw new Error("broken native package unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("Failed to load native binding:")) process.exit(1);
  if (!error.message.includes("GLIBC_2.39")) process.exit(1);
  if (error.message.includes("optional dependencies")) process.exit(1);
  if (!hasCauseCode(error, "ERR_DLOPEN_FAILED")) process.exit(1);
  console.log("package root surfaced dlopen failure");
}
`,
    );
    assertSucceeded(rejected, "require('@vizejs/native') with broken native package");
    assert.match(rejected.stdout, /surfaced dlopen failure/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("packed package root surfaces missing internal dependencies", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-internal-"));
  try {
    const installDir = installPackedNativePackage(tempDir);
    installFakeTargetPackage(installDir, 'require("missing-internal-native-helper");\n');
    const rejected = runProbe(
      installDir,
      `
try {
  require("@vizejs/native");
  throw new Error("broken native package unexpectedly loaded");
} catch (error) {
  if (!error.message.includes("missing-internal-native-helper")) process.exit(1);
  if (error.message.includes("optional dependencies")) process.exit(1);
  console.log("package root surfaced missing internal dependency");
}
`,
    );
    assertSucceeded(rejected, "require('@vizejs/native') with missing internal dependency");
    assert.match(rejected.stdout, /surfaced missing internal dependency/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
