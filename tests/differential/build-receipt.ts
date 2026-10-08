import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "./sha256.ts";

export const BUILD_RECIPE = "cargo build --profile ci -p vize";
export const SHIPPING_BUILD_RECIPE = "cargo build --release -p vize";
// Source launchers retain their owned target/ci staging path; paired build
// custody attests that this is a byte-exact copy of the actual release ELF.
export const LEGACY_BUILD_RECIPE = `${BUILD_RECIPE} --features legacy`;

export type BuildRecipe =
  | typeof BUILD_RECIPE
  | typeof LEGACY_BUILD_RECIPE
  | typeof SHIPPING_BUILD_RECIPE;

/** Complete source/executable identity; additional receipt fields stay opaque. */
export interface BuildIdentity {
  sourceRevision: string;
  binaryPath: string;
  binarySha256: string;
  cliVersion: string;
  [field: string]: unknown;
}

export interface BuildReceipt extends BuildIdentity {
  schema: "vize.differential.build";
  version: 1;
  recipe: BuildRecipe;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object";
}

function validateBuildRecipe(recipe: unknown): asserts recipe is BuildRecipe {
  assert.ok(
    recipe === BUILD_RECIPE || recipe === LEGACY_BUILD_RECIPE || recipe === SHIPPING_BUILD_RECIPE,
    "unknown source-build recipe",
  );
}
export const binaryRelativePath = (): string =>
  `target/ci/${process.platform === "win32" ? "vize.exe" : "vize"}`;

export function sourceRevision(repoRoot: string): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  const revision = result.stdout.trim();
  assert.match(revision, /^[a-f0-9]{40}$/);
  return revision;
}

export function validateBuildReceipt(
  receipt: unknown,
  expected: BuildIdentity,
  recipe: unknown = BUILD_RECIPE,
): asserts receipt is BuildReceipt {
  validateBuildRecipe(recipe);
  assert.ok(isRecord(receipt), "source-build receipt object is required");
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.recipe, recipe);
  assert.ok(typeof receipt.sourceRevision === "string", "source-build source revision is required");
  assert.match(receipt.sourceRevision, /^[a-f0-9]{40}$/);
  assert.ok(typeof receipt.binarySha256 === "string", "source-build binary hash is required");
  assert.match(receipt.binarySha256, /^[a-f0-9]{64}$/);
  assert.ok(typeof receipt.cliVersion === "string", "source-build CLI version is required");
  assert.match(receipt.cliVersion, /^vize \d+\.\d+\.\d+$/);
  assert.ok(typeof receipt.binaryPath === "string", "source-build binary path is required");
  for (const field of ["sourceRevision", "binaryPath", "binarySha256", "cliVersion"]) {
    assert.equal(receipt[field], expected[field], `source-build receipt mismatch: ${field}`);
  }
}

export function expectedBuildIdentity(repoRoot: string): BuildIdentity {
  const binaryPath = binaryRelativePath();
  const cargo = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  const version = cargo.match(/\[workspace\.package\][\s\S]*?^version = "([^"]+)"/m);
  assert(version, "workspace package version is required");
  return {
    sourceRevision: sourceRevision(repoRoot),
    binaryPath,
    binarySha256: sha256(fs.readFileSync(path.join(repoRoot, binaryPath))),
    cliVersion: `vize ${version[1]}`,
  };
}

// Called immediately after a successful source build by the existing tooling jobs.
// This receipt records that CI build; it never runs or builds the CLI itself.
export function writeBuildReceipt(repoRoot: string, recipe: unknown = BUILD_RECIPE): string {
  validateBuildRecipe(recipe);
  const receipt: BuildReceipt = {
    schema: "vize.differential.build",
    version: 1,
    recipe,
    ...expectedBuildIdentity(repoRoot),
  };
  const destination = path.join(repoRoot, `${receipt.binaryPath}.differential-build.json`);
  fs.writeFileSync(destination, `${JSON.stringify(receipt, null, 2)}\n`);
  return destination;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  assert.ok(args.length === 0 || (args.length === 1 && args[0] === "--legacy"));
  console.log(
    writeBuildReceipt(
      path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
      args.length ? LEGACY_BUILD_RECIPE : BUILD_RECIPE,
    ),
  );
}
