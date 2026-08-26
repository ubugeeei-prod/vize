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

function s2DomWitnessFiles(): string[] {
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

function assertS0AliasConsumer(options: {
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
