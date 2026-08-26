import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const lspClientModule = path.join(repoRoot, "crates", "vize_canon", "src", "lsp_client.rs");
const lspClientRoot = path.join(repoRoot, "crates", "vize_canon", "src", "lsp_client");
const require = createRequire(import.meta.url);
const toml = require("@iarna/toml") as { parse(source: string): unknown };

interface CargoDependency {
  readonly name: string;
  readonly rename: string | null;
  readonly path: string | null;
  readonly kind: string | null;
  readonly optional: boolean;
  readonly uses_default_features: boolean;
  readonly req: string;
}

interface CargoPackage {
  readonly name: string;
  readonly version: string;
  readonly dependencies: readonly CargoDependency[];
}

interface CargoMetadata {
  readonly packages: readonly CargoPackage[];
}

function* rustFiles(directory: string): Generator<string> {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      yield* rustFiles(fullPath);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      yield fullPath;
    }
  }
}

function asRecord(value: unknown): Record<string, unknown> {
  assert.equal(typeof value, "object");
  assert.notEqual(value, null);
  assert.ok(!Array.isArray(value));
  return value as Record<string, unknown>;
}

function readToml(...segments: string[]): Record<string, unknown> {
  return asRecord(toml.parse(fs.readFileSync(path.join(repoRoot, ...segments), "utf8")));
}

function cargoMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
}

test("Canon LSP client imports S0 storage through the stage alias", () => {
  const rootManifest = readToml("Cargo.toml");
  const canonManifest = readToml("crates", "vize_canon", "Cargo.toml");
  const workspaceDependencies = asRecord(asRecord(rootManifest.workspace).dependencies);
  const canonDependencies = asRecord(canonManifest.dependencies);
  const workspaceS0 = asRecord(workspaceDependencies.vize_s0);
  const canonS0 = asRecord(canonDependencies.vize_s0);

  assert.equal(workspaceS0.package, "vize_carton");
  assert.equal(workspaceS0.path, "crates/vize_carton");
  assert.equal(canonS0.workspace, true);
  assert.ok(!Object.hasOwn(canonDependencies, "vize_carton"));

  const metadata = cargoMetadata();
  const cartonPackage = metadata.packages.find((pkg) => pkg.name === "vize_carton");
  const canonPackage = metadata.packages.find((pkg) => pkg.name === "vize_canon");
  assert.ok(cartonPackage);
  assert.ok(canonPackage);
  const s0Dependency = canonPackage.dependencies.find(
    (dependency) => dependency.name === "vize_carton" && dependency.rename === "vize_s0",
  );
  assert.ok(s0Dependency);
  assert.equal(s0Dependency.path, path.join(repoRoot, "crates", "vize_carton"));
  assert.equal(s0Dependency.req, `=${cartonPackage.version}`);
  assert.equal(s0Dependency.kind, null);
  assert.equal(s0Dependency.optional, false);
  assert.equal(s0Dependency.uses_default_features, true);

  const offenders = [];
  let aliasImportCount = 0;
  for (const file of [lspClientModule, ...rustFiles(lspClientRoot)]) {
    const source = fs.readFileSync(file, "utf8");
    if (/\bvize_carton::|use vize_carton\b/u.test(source)) {
      offenders.push(path.relative(repoRoot, file));
    }
    if (/\bvize_s0::|use vize_s0\b/u.test(source)) {
      aliasImportCount += 1;
    }
  }

  assert.deepEqual(offenders, []);
  assert.ok(aliasImportCount > 0, "lsp_client should use vize_s0");
});
