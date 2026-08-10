import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { CORSA_BIN } from "../../_helpers/apps.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const fixture = path.join(root, "tests/_fixtures/_projects/typescript-go-module-resolution-race");
const tsconfig = path.join(fixture, "tsconfig.json");
const nativePreviewManifest = path.join(
  root,
  "node_modules/@typescript/native-preview/package.json",
);
const tscEntry = path.join(root, "tests/node_modules/typescript/bin/tsc");
// 20260602.1 (gitHead fe932f69) predates microsoft/typescript-go#4178 and loses
// random package resolutions under the default concurrent loader. This is the
// first published nightly whose history contains that upstream fix.
const fixedVersion = "7.0.0-dev.20260603.1";
const fixedGitHead = "67c7f9dc019913f7227326f7edf576e1180970ea";
const fixedPlatformPackages = [
  "@typescript/native-preview-darwin-arm64",
  "@typescript/native-preview-darwin-x64",
  "@typescript/native-preview-linux-arm",
  "@typescript/native-preview-linux-arm64",
  "@typescript/native-preview-linux-x64",
  "@typescript/native-preview-win32-arm64",
  "@typescript/native-preview-win32-x64",
];
const repeats = 100;

test("the pinned tsgo contains the upstream realpath race fix", () => {
  const manifest = JSON.parse(fs.readFileSync(nativePreviewManifest, "utf8")) as {
    gitHead?: string;
    optionalDependencies?: Record<string, string>;
    version?: string;
  };
  assert.deepEqual(
    { gitHead: manifest.gitHead, version: manifest.version },
    { gitHead: fixedGitHead, version: fixedVersion },
  );
  assert.deepEqual(
    manifest.optionalDependencies,
    Object.fromEntries(fixedPlatformPackages.map((name) => [name, fixedVersion])),
  );

  const version = spawnSync(CORSA_BIN, ["--version"], { encoding: "utf8" });
  assert.equal(version.error, undefined);
  assert.equal(version.signal, null);
  assert.equal(version.status, 0, version.stderr);
  assert.match(version.stdout, new RegExp(`Version ${fixedVersion.replaceAll(".", "\\.")}`));
});

test(
  "default-concurrent tsgo resolves the same package graph in 100 fresh processes",
  { timeout: 120_000 },
  () => {
    const expected = runCompiler(CORSA_BIN, ["-p", tsconfig, "--pretty", "false"]);
    for (let run = 2; run <= repeats; run += 1) {
      assert.deepEqual(
        runCompiler(CORSA_BIN, ["-p", tsconfig, "--pretty", "false"]),
        expected,
        `tsgo module resolution changed on fresh process ${run}/${repeats}`,
      );
    }
  },
);

test("JavaScript tsc agrees that the package graph resolves without diagnostics", () => {
  runCompiler(process.execPath, [tscEntry, "-p", tsconfig, "--pretty", "false"]);
});

function runCompiler(command: string, args: string[]): { stderr: string; stdout: string } {
  const result = spawnSync(command, args, {
    cwd: fixture,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    timeout: 30_000,
  });
  const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
  assert.equal(result.error, undefined, String(result.error));
  assert.equal(result.signal, null, output);
  assert.equal(result.status, 0, output);
  assert.doesNotMatch(output, /TS(?:2307|2882)/);
  assert.equal(output, "", output);
  return { stderr: result.stderr, stdout: result.stdout };
}
