import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Dependency = {
  name: string;
  rename: string | null;
  kind: "dev" | "build" | null;
};

type Package = {
  name: string;
  dependencies: Dependency[];
};

type Metadata = { packages: Package[] };

function readMetadata(): Metadata {
  const result = spawnSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--locked"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as Metadata;
}

function workspacePackage(metadata: Metadata, name: string): Package {
  const found = metadata.packages.find((pkg) => pkg.name === name);
  assert.ok(found, `workspace package ${name} must exist`);
  return found;
}

function dependency(
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

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

function* walkRustFiles(directory: string): Generator<string> {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      yield* walkRustFiles(fullPath);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      yield fullPath;
    }
  }
}

const aliases = new Map<string, ReadonlyArray<readonly [string, string | null]>>([
  ["vize_davinci", [["vize_carton", "vize_s0"]]],
  ["vize_s1", [["vize_carton", "vize_s0"]]],
  ["vize_s2", [["vize_carton", "vize_s0"]]],
  [
    "vize_ricalco",
    [
      ["vize_carton", "vize_s0"],
      ["vize_s1", null],
      ["vize_s2", null],
    ],
  ],
]);

const metadata = readMetadata();

test("Davinci crates import retained packages through stage aliases", () => {
  for (const [packageName, expectedAliases] of aliases) {
    const dependencies = workspacePackage(metadata, packageName).dependencies;
    for (const [dependencyName, rename] of expectedAliases) {
      assert.ok(
        dependencies.some(
          (dependency) =>
            dependency.kind === null &&
            dependency.name === dependencyName &&
            dependency.rename === rename,
        ),
        `${packageName} must import ${dependencyName} as ${rename}`,
      );
    }
  }
});

test("Davinci stage dependencies are one-way and acyclic", () => {
  const tiers = new Map<string, number>([
    ["vize_carton", 0],
    ["vize_davinci", 1],
    ["vize_s1", 1],
    ["vize_s2", 2],
    ["vize_ricalco", 3],
  ]);
  const expectedEdges = new Map<string, string[]>([
    ["vize_carton", []],
    ["vize_davinci", ["vize_carton"]],
    ["vize_s1", ["vize_carton"]],
    ["vize_s2", ["vize_carton", "vize_davinci"]],
    ["vize_ricalco", ["vize_carton", "vize_davinci", "vize_s1", "vize_s2"]],
  ]);

  for (const [packageName, packageTier] of tiers) {
    const dependencies = workspacePackage(metadata, packageName).dependencies;
    const stageEdges = dependencies
      .filter((dependency) => dependency.kind === null && tiers.has(dependency.name))
      .map((dependency) => dependency.name)
      .sort();
    assert.deepEqual(stageEdges, expectedEdges.get(packageName));

    for (const dependency of dependencies) {
      if (dependency.kind !== null) continue;
      const dependencyTier = tiers.get(dependency.name);
      if (dependencyTier === undefined) continue;
      assert.ok(
        dependencyTier < packageTier,
        `${packageName} (tier ${packageTier}) reverses the edge to ${dependency.name} ` +
          `(tier ${dependencyTier})`,
      );
    }
  }
});

test("Davinci fuzz harness imports stage packages through aliases", () => {
  const manifest = readRepoFile("tests", "fuzz", "Cargo.toml");
  assert.match(
    manifest,
    /^vize_s0 = \{ package = "vize_carton", path = "\.\.\/\.\.\/crates\/vize_carton" \}$/m,
  );
  assert.match(
    manifest,
    /^vize_s1_to_s2 = \{ package = "vize_ricalco", path = "\.\.\/\.\.\/crates\/vize_ricalco" \}$/m,
  );
  assert.match(manifest, /^vize_s2 = \{ path = "\.\.\/\.\.\/crates\/vize_disegno" \}$/m);
  assert.doesNotMatch(manifest, /^vize_(?:carton|disegno|ricalco) = /m);

  for (const target of ["folio_parse.rs", "s1_lowering.rs", "template_compile.rs"]) {
    const source = readRepoFile("tests", "fuzz", "fuzz_targets", target);
    assert.doesNotMatch(source, /\bvize_(?:carton|disegno|ricalco)::/u);
  }
});

test("Davinci DOM lane tests import lowering through the stage alias", () => {
  const lowering = dependency(metadata, "vize_atelier_dom", "vize_ricalco", "dev");
  assert.equal(lowering.rename, "vize_s1_to_s2");
  const dependencies = workspacePackage(metadata, "vize_atelier_dom").dependencies;
  assert.ok(
    dependencies.every(
      (dependency) =>
        dependency.name !== "vize_ricalco" ||
        (dependency.kind === "dev" && dependency.rename === "vize_s1_to_s2"),
    ),
    "vize_atelier_dom must not depend on vize_ricalco through its physical name",
  );

  for (const file of [
    "davinci_s2_dom.rs",
    "davinci_s2_patch_flags.rs",
    "davinci_s2_slots.rs",
    path.join("support", "mod.rs"),
  ]) {
    const source = readRepoFile("crates", "vize_atelier_dom", "tests", file);
    assert.doesNotMatch(source, /\bvize_ricalco::/u);
  }
});

test("Vize CLI package imports S0 storage through the stage alias", () => {
  const dependencies = workspacePackage(metadata, "vize").dependencies;
  assert.ok(
    dependencies.some(
      (dependency) =>
        dependency.kind === null &&
        dependency.name === "vize_carton" &&
        dependency.rename === "vize_s0",
    ),
    "vize must import vize_carton as vize_s0 for S0 storage",
  );
  assert.ok(
    dependencies.every(
      (dependency) => dependency.name !== "vize_carton" || dependency.rename === "vize_s0",
    ),
    "vize must not depend on vize_carton through its physical name",
  );

  const vizeDir = path.join(repoRoot, "crates", "vize");
  const offenders = [];
  let aliasImports = 0;
  for (const fullPath of walkRustFiles(vizeDir)) {
    const source = fs.readFileSync(fullPath, "utf8");
    if (/\bvize_carton::/u.test(source)) {
      offenders.push(path.relative(repoRoot, fullPath));
    }
    if (/\bvize_s0::|use vize_s0\b/u.test(source)) {
      aliasImports += 1;
    }
  }

  assert.ok(aliasImports > 0, "vize package should use the vize_s0 alias");
  assert.deepEqual(offenders, []);
});
