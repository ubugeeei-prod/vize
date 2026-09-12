import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  assertS0AliasConsumer,
  dependency,
  metadata,
  readRepoFile,
  repoRoot,
  s2DomWitnessFiles,
  walkRustFiles,
  workspaceDependencyDeclaration,
  workspacePackage,
} from "./support/davinci-stage-dependencies.ts";

const aliases = new Map<string, ReadonlyArray<readonly [string, string | null]>>([
  ["vize_davinci", [["vize_carton", "vize_s0"]]],
  ["vize_s1", [["vize_carton", "vize_s0"]]],
  ["vize_s2", [["vize_carton", "vize_s0"]]],
  [
    "vize_impeto",
    [
      ["vize_carton", "vize_s0"],
      ["vize_davinci", null],
    ],
  ],
  [
    "vize_s1_to_s2",
    [
      ["vize_carton", "vize_s0"],
      ["vize_s1", null],
      ["vize_s2", null],
    ],
  ],
]);

const publishedDavinciStages = new Set([
  "vize_davinci_derive",
  "vize_davinci",
  "vize_s1",
  "vize_s2",
  "vize_impeto",
  "vize_s1_to_s2",
]);

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
    ["vize_impeto", 3],
    ["vize_s1_to_s2", 3],
  ]);
  const expectedEdges = new Map<string, string[]>([
    ["vize_carton", []],
    ["vize_davinci", ["vize_carton"]],
    ["vize_s1", ["vize_carton"]],
    ["vize_s2", ["vize_carton", "vize_davinci"]],
    ["vize_impeto", ["vize_carton", "vize_davinci"]],
    ["vize_s1_to_s2", ["vize_carton", "vize_davinci", "vize_s1", "vize_s2"]],
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

test("Davinci stage crates are publishable with registry-resolvable dependencies", () => {
  for (const packageName of publishedDavinciStages) {
    const pkg = workspacePackage(metadata, packageName);
    assert.equal(pkg.publish, null, `${packageName} must be publishable for the production switch`);
    for (const dependency of pkg.dependencies) {
      if (dependency.kind === "dev" || !publishedDavinciStages.has(dependency.name)) continue;
      assert.match(
        dependency.req,
        /^=\d+\.\d+\.\d+$/u,
        `${packageName} must give ${dependency.name} an exact registry fallback`,
      );
      assert.equal(
        dependency.req,
        `=${workspacePackage(metadata, dependency.name).version}`,
        `${packageName} must match ${dependency.name}'s published version`,
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
  assert.match(manifest, /^vize_s1_to_s2 = \{ path = "\.\.\/\.\.\/crates\/vize_s1_to_s2" \}$/m);
  assert.match(manifest, /^vize_s2 = \{ path = "\.\.\/\.\.\/crates\/vize_s2" \}$/m);
  assert.doesNotMatch(manifest, /^vize_(?:carton|disegno|ricalco) = /m);

  for (const target of ["folio_parse.rs", "s1_lowering.rs", "template_compile.rs"]) {
    const source = readRepoFile("tests", "fuzz", "fuzz_targets", target);
    assert.doesNotMatch(source, /\bvize_(?:carton|disegno|ricalco)::/u);
  }
});

test("Davinci S2 uses the physical crate directory", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_s2",$/m);
  assert.deepEqual(workspaceDependencyDeclaration("vize_s2"), {
    path: "crates/vize_s2",
    version: `=${workspacePackage(metadata, "vize_s2").version}`,
  });
  assert.doesNotMatch(workspaceManifest, /crates\/vize_disegno/u);
});

test("Davinci S1-to-S2 uses the physical crate package and directory", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_s1_to_s2",$/m);
  assert.deepEqual(workspaceDependencyDeclaration("vize_s1_to_s2"), {
    path: "crates/vize_s1_to_s2",
    version: `=${workspacePackage(metadata, "vize_s1_to_s2").version}`,
  });
  assert.doesNotMatch(workspaceManifest, /crates\/vize_ricalco/u);
  assert.doesNotMatch(workspaceManifest, /^vize_ricalco = /m);
  assert.doesNotMatch(workspaceManifest, /package = "vize_ricalco"/u);

  const loweringManifest = readRepoFile("crates", "vize_s1_to_s2", "Cargo.toml");
  assert.match(loweringManifest, /^name = "vize_s1_to_s2"$/m);

  const lockfile = readRepoFile("Cargo.lock");
  assert.match(lockfile, /^name = "vize_s1_to_s2"$/m);
  assert.doesNotMatch(lockfile, /\bvize_ricalco\b/u);
});

test("Davinci S3 uses the Impeto package through the stage alias", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_impeto",$/m);
  assert.deepEqual(workspaceDependencyDeclaration("vize_s3"), {
    path: "crates/vize_impeto",
    version: `=${workspacePackage(metadata, "vize_impeto").version}`,
  });
  assert.doesNotMatch(workspaceManifest, /^vize_s3 = \{ path = "crates\/vize_s3"/m);

  const impetoManifest = readRepoFile("crates", "vize_impeto", "Cargo.toml");
  assert.match(impetoManifest, /^name = "vize_impeto"$/m);
});

test("Davinci S1-to-S2 source paths use the physical S2 folio type", () => {
  const sourceDir = path.join(repoRoot, "crates", "vize_s1_to_s2", "src");
  for (const fullPath of walkRustFiles(sourceDir)) {
    const source = fs.readFileSync(fullPath, "utf8");
    assert.doesNotMatch(
      source,
      /\bDisegnoFolio\b/u,
      `${path.relative(sourceDir, fullPath)} must use S2Folio`,
    );
  }
});

test("Davinci DOM production imports lowering through the physical S1-to-S2 package", () => {
  const lowering = dependency(metadata, "vize_atelier_dom", "vize_s1_to_s2", null);
  assert.equal(lowering.rename, null);
  const dependencies = workspacePackage(metadata, "vize_atelier_dom").dependencies;
  assert.ok(
    dependencies.every(
      (dependency) =>
        dependency.name !== "vize_s1_to_s2" ||
        (dependency.kind === null && dependency.rename === null),
    ),
    "vize_atelier_dom must use the physical vize_s1_to_s2 package name in production",
  );

  for (const file of s2DomWitnessFiles()) {
    const source = readRepoFile("crates", "vize_atelier_dom", "tests", file);
    assert.doesNotMatch(source, /\bvize_ricalco::/u, `${file} must use vize_s1_to_s2`);
  }
});

test("Atelier core S2 witnesses import lowering through the physical S1-to-S2 package", () => {
  const lowering = dependency(metadata, "vize_atelier_core", "vize_s1_to_s2", "dev");
  assert.equal(lowering.rename, null);
  const dependencies = workspacePackage(metadata, "vize_atelier_core").dependencies;
  assert.ok(
    dependencies.every(
      (dependency) =>
        dependency.name !== "vize_s1_to_s2" ||
        (dependency.kind === "dev" && dependency.rename === null),
    ),
    "vize_atelier_core must use the physical vize_s1_to_s2 package name",
  );

  const testDir = path.join(repoRoot, "crates", "vize_atelier_core", "tests");
  for (const fullPath of walkRustFiles(testDir)) {
    const source = fs.readFileSync(fullPath, "utf8");
    assert.doesNotMatch(
      source,
      /\bvize_ricalco::/u,
      `${path.relative(testDir, fullPath)} must use vize_s1_to_s2`,
    );
  }
});

const s0AliasConsumers = [
  ["vize", "vize package", ["crates", "vize"]],
  ["vize_test_runner", "Test runner", ["tests", "vize_test_runner"]],
  ["vize_armature", "Armature parser", ["crates", "vize_armature"]],
  ["vize_patina", "Patina linter", ["crates", "vize_patina"]],
  ["vize_musea", "Musea component gallery", ["crates", "vize_musea"]],
  ["vize_fresco", "Fresco TUI", ["crates", "vize_fresco"]],
  ["vize_curator", "Curator reporting utilities", ["crates", "vize_curator"]],
  ["vize_vitrine", "Vitrine bindings", ["crates", "vize_vitrine"]],
  ["vize_maestro", "Maestro LSP", ["crates", "vize_maestro"]],
  ["vize_atelier_ssr", "Atelier SSR compiler", ["crates", "vize_atelier_ssr"]],
  ["vize_atelier_dom", "Atelier DOM compiler", ["crates", "vize_atelier_dom"]],
  ["vize_atelier_jsx", "Atelier JSX compiler", ["crates", "vize_atelier_jsx"]],
  ["vize_relief", "Relief AST", ["crates", "vize_relief"]],
] as const;

for (const [packageName, label, parts] of s0AliasConsumers) {
  test(`${label} imports S0 storage through the stage alias`, () => {
    assertS0AliasConsumer({ packageName, label, directory: path.join(repoRoot, ...parts) });
  });
}

test("Canon content-mapper imports S0 storage through the stage alias", () => {
  const manifest = readRepoFile("crates", "vize_canon", "Cargo.toml");
  assert.match(manifest, /^vize_s0\.workspace = true$/m);
  assert.doesNotMatch(manifest, /^vize_carton\.workspace = true$/m);
  assertS0AliasConsumer({
    packageName: "vize_canon",
    label: "Canon content-mapper",
    directory: path.join(repoRoot, "crates", "vize_canon", "src", "batch", "virtual_project"),
    filter: (fullPath) => path.basename(fullPath).startsWith("content_mapper"),
  });
});

test("Atelier core compiler macros import S0 storage through the stage alias", () => {
  const manifest = readRepoFile("crates", "vize_atelier_core", "Cargo.toml");
  assert.match(manifest, /^vize_s0 = \{ workspace = true \}$/m);
  assert.doesNotMatch(manifest, /^vize_carton\.workspace = true$/m);
  assertS0AliasConsumer({
    packageName: "vize_atelier_core",
    label: "Atelier core compiler",
    directory: path.join(repoRoot, "crates", "vize_atelier_core", "src"),
  });

  for (const relative of ["lib.rs", "test_macros.rs"]) {
    const source = readRepoFile("crates", "vize_atelier_core", "src", relative);
    assert.doesNotMatch(source, /\bvize_carton\b/u, `${relative} must use vize_s0 or $crate`);
  }
});
