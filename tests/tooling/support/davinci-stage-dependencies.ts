import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseToml } from "@iarna/toml";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export type Dependency = {
  name: string;
  features: string[];
  rename: string | null;
  kind: "dev" | "build" | null;
  optional: boolean;
  req: string;
};

export type Package = {
  name: string;
  dependencies: Dependency[];
  features: Record<string, string[]>;
  manifest_path: string;
  publish: string[] | null;
  version: string;
};

export type Metadata = { packages: Package[] };

export const metadata = readMetadata();

export function readMetadata(): Metadata {
  const result = spawnSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--locked"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as Metadata;
}

export function workspacePackage(metadata: Metadata, name: string): Package {
  const found = metadata.packages.find((pkg) => pkg.name === name);
  assert.ok(found, `workspace package ${name} must exist`);
  return found;
}

export function dependency(
  metadata: Metadata,
  packageName: string,
  dependencyName: string,
  kind: Dependency["kind"],
): Dependency {
  const found = workspacePackage(metadata, packageName).dependencies.find(
    (dependency) => dependency.kind === kind && dependency.name === dependencyName,
  );
  assert.ok(found, `${packageName} must declare ${kind ?? "normal"} dependency ${dependencyName}`);
  return found;
}

export function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

export function workspaceDependencyDeclaration(name: string): {
  path: string;
  version: string;
} {
  const manifest = parseToml(readRepoFile("Cargo.toml")) as {
    workspace?: { dependencies?: Record<string, unknown> };
  };
  const declaration = manifest.workspace?.dependencies?.[name];
  assert.ok(
    declaration !== null && typeof declaration === "object" && !Array.isArray(declaration),
    `workspace dependency ${name} must be a table`,
  );

  const { path: dependencyPath, version } = declaration as Record<string, unknown>;
  assert.equal(typeof dependencyPath, "string", `${name} must declare a path`);
  assert.equal(typeof version, "string", `${name} must declare a version`);
  return { path: dependencyPath, version };
}

export function* walkRustFiles(directory: string): Generator<string> {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      yield* walkRustFiles(fullPath);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      yield fullPath;
    }
  }
}

export function s2DomWitnessFiles(): string[] {
  const testDir = path.join(repoRoot, "crates", "vize_atelier_dom", "tests");
  const witnesses = fs
    .readdirSync(testDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && /^davinci_s2_.*\.rs$/u.test(entry.name))
    .map((entry) => entry.name)
    .sort();
  assert.ok(
    witnesses.length >= 20,
    `expected broad S2 DOM witness coverage, found only ${witnesses.length} files`,
  );
  return [...witnesses, path.join("support", "mod.rs")];
}

export function assertS0AliasConsumer(options: {
  packageName: string;
  label: string;
  directory: string;
  filter?: (fullPath: string) => boolean;
}) {
  const { packageName, label, directory, filter = () => true } = options;
  const dependencies = workspacePackage(metadata, packageName).dependencies;
  assert.ok(
    dependencies.some(
      (dependency) =>
        dependency.kind === null &&
        dependency.name === "vize_carton" &&
        dependency.rename === "vize_s0",
    ),
    `${packageName} must import vize_carton as vize_s0 for S0 storage`,
  );
  assert.ok(
    dependencies.every(
      (dependency) => dependency.name !== "vize_carton" || dependency.rename === "vize_s0",
    ),
    `${packageName} must not depend on vize_carton through its physical name`,
  );

  const offenders = [];
  let aliasImports = 0;
  for (const fullPath of walkRustFiles(directory)) {
    if (!filter(fullPath)) continue;
    const source = fs.readFileSync(fullPath, "utf8");
    if (/\bvize_carton::|use vize_carton\b/u.test(source)) {
      offenders.push(path.relative(repoRoot, fullPath));
    }
    if (/\bvize_s0::|use vize_s0\b/u.test(source)) {
      aliasImports += 1;
    }
  }

  assert.ok(aliasImports > 0, `${label} should use the vize_s0 alias`);
  assert.deepEqual(offenders, []);
}
