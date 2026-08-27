import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const libPath = path.join(repoRoot, "crates", "vize_atelier_core", "src", "lib.rs");
const slotsModule = path.join(
  repoRoot,
  "crates",
  "vize_atelier_core",
  "src",
  "codegen",
  "slots.rs",
);
const propsModule = path.join(
  repoRoot,
  "crates",
  "vize_atelier_core",
  "src",
  "codegen",
  "props.rs",
);
const vIfModule = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "v_if.rs");
const vForModule = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "v_for.rs");
const contextModule = path.join(
  repoRoot,
  "crates",
  "vize_atelier_core",
  "src",
  "codegen",
  "context.rs",
);
const componentBindingModule = path.join(
  repoRoot,
  "crates",
  "vize_atelier_core",
  "src",
  "codegen",
  "component_binding.rs",
);
const childrenModule = path.join(
  repoRoot,
  "crates",
  "vize_atelier_core",
  "src",
  "codegen",
  "children.rs",
);
const emitModule = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "emit.rs");
const sourceMapModule = path.join(
  repoRoot,
  "crates",
  "vize_atelier_core",
  "src",
  "codegen",
  "source_map.rs",
);
const slotsRoot = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "slots");
const propsRoot = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "props");
const vIfRoot = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "v_if");
const vForRoot = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "v_for");
const contextRoot = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "context");
const elementRoot = path.join(repoRoot, "crates", "vize_atelier_core", "src", "codegen", "element");
const testRoot = path.join(repoRoot, "crates", "vize_atelier_core", "tests");
const benchPath = path.join(repoRoot, "crates", "vize_atelier_core", "benches", "davinci.rs");
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
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1", "--locked"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
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

test("Atelier core migrated codegen slices import S0 storage through the stage alias", () => {
  const rootManifest = readToml("Cargo.toml");
  const coreManifest = readToml("crates", "vize_atelier_core", "Cargo.toml");
  const workspaceDependencies = asRecord(asRecord(rootManifest.workspace).dependencies);
  const coreDependencies = asRecord(coreManifest.dependencies);
  const workspaceS0 = asRecord(workspaceDependencies.vize_s0);
  const coreS0 = asRecord(coreDependencies.vize_s0);

  assert.equal(workspaceS0.package, "vize_carton");
  assert.equal(workspaceS0.path, "crates/vize_carton");
  assert.equal(coreS0.workspace, true);
  assert.ok(!Object.hasOwn(coreDependencies, "vize_carton"));

  const metadata = cargoMetadata();
  const cartonPackage = metadata.packages.find((pkg) => pkg.name === "vize_carton");
  const corePackage = metadata.packages.find((pkg) => pkg.name === "vize_atelier_core");
  assert.ok(cartonPackage);
  assert.ok(corePackage);
  const s0Dependency = corePackage.dependencies.find(
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
  for (const file of [
    libPath,
    slotsModule,
    propsModule,
    vIfModule,
    vForModule,
    contextModule,
    componentBindingModule,
    childrenModule,
    emitModule,
    sourceMapModule,
    ...rustFiles(slotsRoot),
    ...rustFiles(propsRoot),
    ...rustFiles(vIfRoot),
    ...rustFiles(vForRoot),
    ...rustFiles(contextRoot),
    ...rustFiles(elementRoot),
    ...rustFiles(testRoot),
    benchPath,
  ]) {
    const source = fs.readFileSync(file, "utf8");
    if (/\bvize_carton::|use vize_carton\b/u.test(source)) {
      offenders.push(path.relative(repoRoot, file));
    }
    if (/\bvize_s0::|use vize_s0\b/u.test(source)) {
      aliasImportCount += 1;
    }
  }

  assert.deepEqual(offenders, []);
  assert.ok(aliasImportCount > 0, "migrated codegen slices should use vize_s0");
});
