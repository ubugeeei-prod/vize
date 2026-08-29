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
  workspacePackage,
  type Package,
} from "./support/davinci-stage-dependencies.ts";

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

const unpublishedDavinciStages = new Set(["vize_davinci", "vize_s1", "vize_s2", "vize_ricalco"]);

function isPublishable(pkg: Package): boolean {
  return pkg.publish === null || pkg.publish.length > 0;
}

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

test("Davinci stage crates stay out of published release graphs", () => {
  for (const packageName of unpublishedDavinciStages) {
    assert.deepEqual(
      workspacePackage(metadata, packageName).publish,
      [],
      `${packageName} must stay publish=false until a dedicated stage-publication switch`,
    );
  }

  const offenders: string[] = [];
  const workspacePackages = new Set(metadata.packages.map((pkg) => pkg.name));

  for (const pkg of metadata.packages.filter(isPublishable)) {
    for (const dependency of pkg.dependencies) {
      if (!workspacePackages.has(dependency.name)) continue;
      if (!unpublishedDavinciStages.has(dependency.name)) continue;
      const strippedDevDependency = dependency.kind === "dev" && dependency.req === "*";
      if (strippedDevDependency) continue;

      offenders.push(
        `${pkg.name} ${dependency.kind ?? "normal"} dependency ` +
          `${dependency.rename ?? dependency.name}(${dependency.name}) req ${dependency.req}`,
      );
    }
  }

  assert.deepEqual(offenders, []);
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
  assert.match(workspaceManifest, /^vize_s2 = \{ path = "crates\/vize_s2" \}$/m);
  assert.doesNotMatch(workspaceManifest, /crates\/vize_disegno/u);
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

  for (const file of s2DomWitnessFiles()) {
    const source = readRepoFile("crates", "vize_atelier_dom", "tests", file);
    assert.doesNotMatch(source, /\bvize_ricalco::/u, `${file} must use vize_s1_to_s2`);
  }
});

test("Davinci atelier core S2 witnesses import lowering through the stage alias", () => {
  const lowering = dependency(metadata, "vize_atelier_core", "vize_ricalco", "dev");
  assert.equal(lowering.rename, "vize_s1_to_s2");
  const dependencies = workspacePackage(metadata, "vize_atelier_core").dependencies;
  assert.ok(
    dependencies.every(
      (dependency) =>
        dependency.name !== "vize_ricalco" ||
        (dependency.kind === "dev" && dependency.rename === "vize_s1_to_s2"),
    ),
    "vize_atelier_core must not depend on vize_ricalco through its physical name",
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

test("Vize CLI package imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize",
    label: "vize package",
    directory: path.join(repoRoot, "crates", "vize"),
  });
});

test("Test runner imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_test_runner",
    label: "Test runner",
    directory: path.join(repoRoot, "tests", "vize_test_runner"),
  });
});

test("Armature parser imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_armature",
    label: "Armature parser",
    directory: path.join(repoRoot, "crates", "vize_armature"),
  });
});

test("Patina linter imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_patina",
    label: "Patina linter",
    directory: path.join(repoRoot, "crates", "vize_patina"),
  });
});

test("Fresco TUI imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_fresco",
    label: "Fresco TUI",
    directory: path.join(repoRoot, "crates", "vize_fresco"),
  });
});

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

test("Curator reporting utilities import S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_curator",
    label: "Curator reporting utilities",
    directory: path.join(repoRoot, "crates", "vize_curator"),
  });
});

test("Maestro LSP imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_maestro",
    label: "Maestro LSP",
    directory: path.join(repoRoot, "crates", "vize_maestro"),
  });
});

test("Atelier SSR compiler imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_atelier_ssr",
    label: "Atelier SSR compiler",
    directory: path.join(repoRoot, "crates", "vize_atelier_ssr"),
  });
});

test("Atelier DOM compiler imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_atelier_dom",
    label: "Atelier DOM compiler",
    directory: path.join(repoRoot, "crates", "vize_atelier_dom"),
  });
});

test("Relief AST imports S0 storage through the stage alias", () => {
  assertS0AliasConsumer({
    packageName: "vize_relief",
    label: "Relief AST",
    directory: path.join(repoRoot, "crates", "vize_relief"),
  });
});
