import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  assertL0AliasConsumer,
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
  ["vize_davinci", [["vize_carton", "vize_l0"]]],
  ["vize_l1", [["vize_carton", "vize_l0"]]],
  ["vize_l2", [["vize_carton", "vize_l0"]]],
  [
    "vize_impeto",
    [
      ["vize_carton", "vize_l0"],
      ["vize_davinci", null],
    ],
  ],
  [
    "vize_l1_to_l2",
    [
      ["vize_carton", "vize_l0"],
      ["vize_l1", null],
      ["vize_impeto", null],
      ["vize_l2", null],
    ],
  ],
  [
    "vize_l2_to_l3",
    [
      ["vize_carton", "vize_l0"],
      ["vize_l2", null],
      ["vize_impeto", "vize_l3"],
    ],
  ],
]);

const publishedDavinciStages = new Set([
  "vize_davinci_derive",
  "vize_davinci",
  "vize_l1",
  "vize_l2",
  "vize_impeto",
  "vize_l1_to_l2",
  "vize_l2_to_l3",
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
    ["vize_l1", 1],
    ["vize_l2", 2],
    ["vize_impeto", 3],
    ["vize_l1_to_l2", 4],
    ["vize_l2_to_l3", 4],
  ]);
  const expectedEdges = new Map<string, string[]>([
    ["vize_carton", []],
    ["vize_davinci", ["vize_carton"]],
    ["vize_l1", ["vize_carton"]],
    ["vize_l2", ["vize_carton", "vize_davinci"]],
    ["vize_impeto", ["vize_carton", "vize_davinci"]],
    ["vize_l1_to_l2", ["vize_carton", "vize_davinci", "vize_impeto", "vize_l1", "vize_l2"]],
    ["vize_l2_to_l3", ["vize_carton", "vize_davinci", "vize_impeto", "vize_l2"]],
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
    /^vize_l0 = \{ package = "vize_carton", path = "\.\.\/\.\.\/crates\/vize_carton" \}$/m,
  );
  assert.match(manifest, /^vize_l1_to_l2 = \{ path = "\.\.\/\.\.\/crates\/vize_l1_to_l2" \}$/m);
  assert.match(manifest, /^vize_l2 = \{ path = "\.\.\/\.\.\/crates\/vize_l2" \}$/m);
  assert.match(manifest, /^vize_l2_to_l3 = \{ path = "\.\.\/\.\.\/crates\/vize_l2_to_l3" \}$/m);
  assert.match(
    manifest,
    /^vize_l3 = \{ package = "vize_impeto", path = "\.\.\/\.\.\/crates\/vize_impeto" \}$/m,
  );
  assert.doesNotMatch(manifest, /^vize_(?:carton|disegno|ricalco) = /m);

  for (const target of [
    "folio_parse.rs",
    "l1_lowering.rs",
    "l2_to_l3_lowering.rs",
    "template_compile.rs",
  ]) {
    const source = readRepoFile("tests", "fuzz", "fuzz_targets", target);
    assert.doesNotMatch(source, /\bvize_(?:carton|disegno|ricalco)::/u);
  }
});

test("Davinci L2 uses the physical crate directory", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_l2",$/m);
  assert.deepEqual(workspaceDependencyDeclaration("vize_l2"), {
    path: "crates/vize_l2",
    version: `=${workspacePackage(metadata, "vize_l2").version}`,
  });
  assert.doesNotMatch(workspaceManifest, /crates\/vize_disegno/u);
});

test("Davinci L1-to-L2 uses the physical crate package and directory", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_l1_to_l2",$/m);
  assert.deepEqual(workspaceDependencyDeclaration("vize_l1_to_l2"), {
    path: "crates/vize_l1_to_l2",
    version: `=${workspacePackage(metadata, "vize_l1_to_l2").version}`,
  });
  assert.doesNotMatch(workspaceManifest, /crates\/vize_ricalco/u);
  assert.doesNotMatch(workspaceManifest, /^vize_ricalco = /m);
  assert.doesNotMatch(workspaceManifest, /package = "vize_ricalco"/u);

  const loweringManifest = readRepoFile("crates", "vize_l1_to_l2", "Cargo.toml");
  assert.match(loweringManifest, /^name = "vize_l1_to_l2"$/m);

  const lockfile = readRepoFile("Cargo.lock");
  assert.match(lockfile, /^name = "vize_l1_to_l2"$/m);
  assert.doesNotMatch(lockfile, /\bvize_ricalco\b/u);
});

test("Davinci L3 uses the Impeto package through the stage alias", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_impeto",$/m);
  assert.deepEqual(workspaceDependencyDeclaration("vize_l3"), {
    path: "crates/vize_impeto",
    version: `=${workspacePackage(metadata, "vize_impeto").version}`,
  });
  assert.doesNotMatch(workspaceManifest, /^vize_l3 = \{ path = "crates\/vize_l3"/m);

  const impetoManifest = readRepoFile("crates", "vize_impeto", "Cargo.toml");
  assert.match(impetoManifest, /^name = "vize_impeto"$/m);
});

test("Davinci L1-to-L2 source paths use the physical L2 folio type", () => {
  const sourceDir = path.join(repoRoot, "crates", "vize_l1_to_l2", "src");
  for (const fullPath of walkRustFiles(sourceDir)) {
    const source = fs.readFileSync(fullPath, "utf8");
    assert.doesNotMatch(
      source,
      /\bDisegnoFolio\b/u,
      `${path.relative(sourceDir, fullPath)} must use L2Folio`,
    );
  }
});

test("Davinci DOM production imports lowering through the physical L1-to-L2 package", () => {
  const lowering = dependency(metadata, "vize_atelier_dom", "vize_l1_to_l2", null);
  assert.equal(lowering.rename, null);
  const dependencies = workspacePackage(metadata, "vize_atelier_dom").dependencies;
  assert.ok(
    dependencies.every(
      (dependency) =>
        dependency.name !== "vize_l1_to_l2" ||
        (dependency.kind === null && dependency.rename === null),
    ),
    "vize_atelier_dom must use the physical vize_l1_to_l2 package name in production",
  );

  for (const file of s2DomWitnessFiles()) {
    const source = readRepoFile("crates", "vize_atelier_dom", "tests", file);
    assert.doesNotMatch(source, /\bvize_ricalco::/u, `${file} must use vize_l1_to_l2`);
  }
});

test("Atelier core L2 witnesses import lowering through the physical L1-to-L2 package", () => {
  const lowering = dependency(metadata, "vize_atelier_core", "vize_l1_to_l2", "dev");
  assert.equal(lowering.rename, null);
  const dependencies = workspacePackage(metadata, "vize_atelier_core").dependencies;
  assert.ok(
    dependencies.every(
      (dependency) =>
        dependency.name !== "vize_l1_to_l2" ||
        (dependency.kind === "dev" && dependency.rename === null),
    ),
    "vize_atelier_core must use the physical vize_l1_to_l2 package name",
  );

  const testDir = path.join(repoRoot, "crates", "vize_atelier_core", "tests");
  for (const fullPath of walkRustFiles(testDir)) {
    const source = fs.readFileSync(fullPath, "utf8");
    assert.doesNotMatch(
      source,
      /\bvize_ricalco::/u,
      `${path.relative(testDir, fullPath)} must use vize_l1_to_l2`,
    );
  }
});

test("Davinci Vapor compile path imports the verified L3 bridge", () => {
  const vapor = workspacePackage(metadata, "vize_atelier_vapor");
  for (const [dependencyName, rename] of [
    ["vize_l1", null],
    ["vize_l1_to_l2", null],
    ["vize_l2_to_l3", null],
    ["vize_impeto", "vize_l3"],
  ] as const) {
    const dep = dependency(metadata, "vize_atelier_vapor", dependencyName, null);
    assert.equal(dep.rename, rename);
    assert.equal(dep.req, `=${workspacePackage(metadata, dependencyName).version}`);
  }

  const compile = readRepoFile("crates", "vize_atelier_vapor", "src", "compile.rs");
  const bridge = readRepoFile("crates", "vize_atelier_vapor", "src", "l3.rs");
  assert.match(compile, /lower_source_for_vapor/u);
  assert.match(bridge, /vize_l1::parse_with_options/u);
  assert.match(bridge, /vize_l1_to_l2::lower/u);
  assert.match(bridge, /vize_l2_to_l3::lower/u);
  assert.match(bridge, /vize_l3::verify::verify/u);
  assert.ok(vapor.dependencies.some((dep) => dep.name === "vize_l2_to_l3"));
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
  test(`${label} imports L0 storage through the stage alias`, () => {
    assertL0AliasConsumer({ packageName, label, directory: path.join(repoRoot, ...parts) });
  });
}

test("Canon content-mapper imports L0 storage through the stage alias", () => {
  const manifest = readRepoFile("crates", "vize_canon", "Cargo.toml");
  assert.match(manifest, /^vize_l0\.workspace = true$/m);
  assert.doesNotMatch(manifest, /^vize_carton\.workspace = true$/m);
  assertL0AliasConsumer({
    packageName: "vize_canon",
    label: "Canon content-mapper",
    directory: path.join(repoRoot, "crates", "vize_canon", "src", "batch", "virtual_project"),
    filter: (fullPath) => path.basename(fullPath).startsWith("content_mapper"),
  });
});

test("Atelier core compiler macros import L0 storage through the stage alias", () => {
  const manifest = readRepoFile("crates", "vize_atelier_core", "Cargo.toml");
  assert.match(manifest, /^vize_l0 = \{ workspace = true \}$/m);
  assert.doesNotMatch(manifest, /^vize_carton\.workspace = true$/m);
  assertL0AliasConsumer({
    packageName: "vize_atelier_core",
    label: "Atelier core compiler",
    directory: path.join(repoRoot, "crates", "vize_atelier_core", "src"),
  });

  for (const relative of ["lib.rs", "test_macros.rs"]) {
    const source = readRepoFile("crates", "vize_atelier_core", "src", relative);
    assert.doesNotMatch(source, /\bvize_carton\b/u, `${relative} must use vize_l0 or $crate`);
  }
});
