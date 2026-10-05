import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "./manifest.mjs";

export const BUILD_RECIPE = "cargo build --profile ci -p vize";
export const LEGACY_BUILD_RECIPE = `${BUILD_RECIPE} --features legacy`;

function validateBuildRecipe(recipe) {
  assert.ok(
    recipe === BUILD_RECIPE || recipe === LEGACY_BUILD_RECIPE,
    "unknown source-build recipe",
  );
}
export const binaryRelativePath = () =>
  `target/ci/${process.platform === "win32" ? "vize.exe" : "vize"}`;

export function sourceRevision(repoRoot) {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  const revision = result.stdout.trim();
  assert.match(revision, /^[a-f0-9]{40}$/);
  return revision;
}

export function validateBuildReceipt(receipt, expected, recipe = BUILD_RECIPE) {
  validateBuildRecipe(recipe);
  assert.equal(receipt.schema, "vize.differential.build");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.recipe, recipe);
  assert.match(receipt.sourceRevision, /^[a-f0-9]{40}$/);
  assert.match(receipt.binarySha256, /^[a-f0-9]{64}$/);
  assert.match(receipt.cliVersion, /^vize \d+\.\d+\.\d+$/);
  for (const field of ["sourceRevision", "binaryPath", "binarySha256", "cliVersion"]) {
    assert.equal(receipt[field], expected[field], `source-build receipt mismatch: ${field}`);
  }
}

export function expectedBuildIdentity(repoRoot) {
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
export function writeBuildReceipt(repoRoot, recipe = BUILD_RECIPE) {
  validateBuildRecipe(recipe);
  const receipt = {
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
