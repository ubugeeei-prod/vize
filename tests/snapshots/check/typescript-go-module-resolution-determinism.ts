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
const runtimePackage = `@typescript/typescript-${process.platform}-${process.arch}`;
const runtimeManifest = path.join(
  root,
  "node_modules",
  "@typescript",
  `typescript-${process.platform}-${process.arch}`,
  "package.json",
);
const tscEntry = path.join(root, "tests/node_modules/typescript/bin/tsc");
const fixedVersion = "7.0.2";
const repeats = 100;

test("the pinned TypeScript runtime is the stable TypeScript 7 package", () => {
  const manifest = JSON.parse(fs.readFileSync(runtimeManifest, "utf8")) as {
    name?: string;
    version?: string;
  };
  assert.deepEqual(
    { name: manifest.name, version: manifest.version },
    { name: runtimePackage, version: fixedVersion },
  );

  const version = spawnSync(CORSA_BIN, ["--version"], { encoding: "utf8" });
  assert.equal(version.error, undefined);
  assert.equal(version.signal, null);
  assert.equal(version.status, 0, version.stderr);
  assert.match(version.stdout, new RegExp(`Version ${fixedVersion.replaceAll(".", "\\.")}`));
});

test(
  "default-concurrent TypeScript resolves the same package graph in 100 fresh processes",
  { timeout: 120_000 },
  () => {
    const expected = runCompiler(CORSA_BIN, ["-p", tsconfig, "--pretty", "false"]);
    for (let run = 2; run <= repeats; run += 1) {
      assert.deepEqual(
        runCompiler(CORSA_BIN, ["-p", tsconfig, "--pretty", "false"]),
        expected,
        `TypeScript module resolution changed on fresh process ${run}/${repeats}`,
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
