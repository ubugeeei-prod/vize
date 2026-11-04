/**
 * First-run failures for the packed init smoke (#11509).
 *
 * Each case has to name the missing package or the action to take, and none of
 * them may continue into type checking. Copies live outside the install tree so
 * a truncated binding or a rewritten version cannot change the healthy cells.
 */

import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { renderOutput, runResult } from "./smoke-process.ts";

const HOST_RUNTIME = ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"];
const TRUNCATED_TARGETS = [
  "darwin-universal",
  "darwin-arm64",
  "darwin-x64",
  "linux-arm64-gnu",
  "linux-arm64-musl",
  "linux-x64-gnu",
  "linux-x64-musl",
  "win32-arm64-msvc",
  "win32-x64-msvc",
];

export interface InitFailureContext {
  readonly installDir?: string | undefined;
  readonly vizeBin?: string | undefined;
}

function failureEnv(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env, ...extra };
  for (const name of HOST_RUNTIME) {
    if (!(name in extra)) delete env[name];
  }
  delete env.VIZE_ALLOW_NATIVE_VERSION_MISMATCH;
  return env;
}

function installedPackageDir(installDir: string | undefined, name: string): string | null {
  if (installDir === undefined) return null;
  const parts = name.split("/");
  const candidates = [
    path.join(installDir, "node_modules", ...parts),
    path.join(installDir, "node_modules", "vize", "node_modules", ...parts),
  ];
  return (
    candidates.find((candidate) => fs.existsSync(path.join(candidate, "package.json"))) ?? null
  );
}

function stagePackage(sourceDir: string, destDir: string): void {
  fs.rmSync(destDir, { force: true, recursive: true });
  fs.mkdirSync(path.dirname(destDir), { recursive: true });
  fs.cpSync(sourceDir, destDir, { recursive: true, dereference: true });
}

function packageBin(packageDir: string, binName: string): string {
  const manifest = JSON.parse(fs.readFileSync(path.join(packageDir, "package.json"), "utf8")) as {
    bin?: string | Readonly<Record<string, string>>;
  };
  const bin = manifest.bin;
  const relative = typeof bin === "string" ? bin : bin?.[binName];
  assert.equal(typeof relative, "string", `${packageDir} does not expose a ${binName} bin`);
  return path.join(packageDir, relative as string);
}

/** A truncated `.node` must throw, naming the file, instead of loading. */
export function assertTruncatedNativePackage(packageDir: string): void {
  for (const target of TRUNCATED_TARGETS) {
    fs.writeFileSync(path.join(packageDir, `vize-vitrine.${target}.node`), "not a native addon");
  }
  const binding = path.join(packageDir, "native-binding.js");
  assert.ok(fs.existsSync(binding), `native binding loader is missing from ${packageDir}`);
  let message = "";
  try {
    createRequire(binding)(binding);
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert.match(message, /Failed to load native binding/);
  assert.match(message, /vize-vitrine\.[A-Za-z0-9-]+\.node/);
  assert.doesNotMatch(message, /disable type checking|typecheck disabled/i);
}

function assertVersionMismatch(vizeDir: string, nativeDir: string): void {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-failure-mismatch-"));
  try {
    const stagedVize = path.join(root, "node_modules", "vize");
    const stagedNative = path.join(root, "node_modules", "@vizejs", "native");
    stagePackage(vizeDir, stagedVize);
    stagePackage(nativeDir, stagedNative);
    const manifestPath = path.join(stagedNative, "package.json");
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as { version?: string };
    manifest.version = "0.0.0-mismatch";
    fs.writeFileSync(manifestPath, `${JSON.stringify(manifest)}\n`);
    const result = runResult(process.execPath, [packageBin(stagedVize, "vize"), "check"], {
      cwd: root,
      env: failureEnv(),
    });
    assert.notEqual(result.status, 0, "mismatched vize and @vizejs/native exited 0");
    const output = renderOutput(result);
    assert.match(output, /vize@\S+ and @vizejs\/native@0\.0\.0-mismatch are different versions/);
    assert.match(output, /Reinstall dependencies so type checking uses one Vize release/);
    assert.doesNotMatch(output, /errorCount/);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

function assertPackageSubdirectory(context: InitFailureContext): void {
  const vizeBin = context.vizeBin;
  assert.equal(typeof vizeBin, "string");
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-failure-subdir-"));
  try {
    const app = path.join(root, "packages", "app");
    const nested = path.join(app, "src");
    fs.mkdirSync(nested, { recursive: true });
    fs.writeFileSync(path.join(app, "package.json"), '{"name":"app","private":true}\n');
    const result = runResult(process.execPath, [vizeBin as string, "init", nested], {
      cwd: root,
      env: failureEnv(),
    });
    assert.notEqual(result.status, 0, "init in a package subdirectory exited 0");
    const output = renderOutput(result);
    assert.match(output, /No package\.json in/);
    assert.match(output, /Run `vize init` from/);
    assert.ok(output.includes(app), output);
    assert.match(output, /Type checking was not configured/);
    assert.deepEqual(fs.readdirSync(nested), []);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

function assertOfflineCache(context: InitFailureContext): void {
  const vizeBin = context.vizeBin;
  assert.equal(typeof vizeBin, "string");
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-failure-offline-"));
  const cache = fs.mkdtempSync(path.join(os.tmpdir(), "vize-failure-npm-cache-"));
  try {
    fs.mkdirSync(path.join(root, ".git"));
    fs.writeFileSync(
      path.join(root, "package.json"),
      `${JSON.stringify({ name: "offline-cache", private: true })}\n`,
    );
    fs.writeFileSync(
      path.join(root, "vite.config.ts"),
      'import { defineConfig } from "vite";\nexport default defineConfig({ plugins: [] });\n',
    );
    const result = runResult(
      process.execPath,
      [vizeBin as string, "init", root, "--yes", "--no-lint", "--vite", "--no-fmt", "--no-editor"],
      {
        cwd: root,
        env: failureEnv({
          npm_config_audit: "false",
          npm_config_cache: cache,
          npm_config_fund: "false",
          npm_config_offline: "true",
          npm_config_update_notifier: "false",
        }),
      },
    );
    assert.notEqual(result.status, 0, "offline init exited 0");
    const output = renderOutput(result);
    assert.match(output, /Type checking was not run/);
    assert.match(output, /Install command failed \(npm install -D .*?\bvize\b/);
    assert.match(output, /ENOTCACHED|only-if-cached/);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
    fs.rmSync(cache, { force: true, recursive: true });
  }
}

/**
 * Runs the four failure cases when the packed install contains both packages.
 *
 * A native-only or CLI-only pack skips the cases instead of failing the job.
 */
export function runInitFailureChecks(context: InitFailureContext): void {
  const nativeDir = installedPackageDir(context.installDir, "@vizejs/native");
  const vizeDir = installedPackageDir(context.installDir, "vize");
  if (nativeDir === null || vizeDir === null || typeof context.vizeBin !== "string") {
    console.log(
      "runtime: packed init failure cases skipped (vize or @vizejs/native not installed)",
    );
    return;
  }
  const truncated = fs.mkdtempSync(path.join(os.tmpdir(), "vize-failure-truncated-"));
  try {
    const staged = path.join(truncated, "native");
    stagePackage(nativeDir, staged);
    assertTruncatedNativePackage(staged);
    assertVersionMismatch(vizeDir, nativeDir);
    assertOfflineCache(context);
    assertPackageSubdirectory(context);
  } finally {
    fs.rmSync(truncated, { force: true, recursive: true });
  }
  console.log(
    "runtime: packed init failure cases (truncated native, version mismatch, offline cache, package subdirectory)",
  );
}
