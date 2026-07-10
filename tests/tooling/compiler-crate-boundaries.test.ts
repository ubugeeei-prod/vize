import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function internalDependencies(crateName: string): string[] {
  const manifest = read(`crates/${crateName}/Cargo.toml`);
  return [...manifest.matchAll(/^vize_([a-z0-9_]+)(?:\s*=|\.)/gm)]
    .map((match) => `vize_${match[1]}`)
    .filter((dependency, index, all) => all.indexOf(dependency) === index)
    .sort();
}

test("compiler foundation ownership is split across physical crates", () => {
  assert.deepEqual(internalDependencies("vize_atlas"), ["vize_carton"]);
  assert.deepEqual(internalDependencies("vize_relief"), ["vize_atlas", "vize_carton"]);
  assert.deepEqual(internalDependencies("vize_croquis"), [
    "vize_armature",
    "vize_atlas",
    "vize_carton",
    "vize_relief",
  ]);
  assert.deepEqual(internalDependencies("vize_croquis_cf"), [
    "vize_armature",
    "vize_atlas",
    "vize_carton",
    "vize_croquis",
  ]);
  assert.deepEqual(internalDependencies("vize_rendu"), ["vize_atlas", "vize_carton"]);
  assert.deepEqual(internalDependencies("vize_flow"), ["vize_atlas", "vize_carton"]);

  const atelierDependencies = internalDependencies("vize_atelier_core");
  for (const dependency of ["vize_atlas", "vize_rendu"]) {
    assert.ok(
      !atelierDependencies.includes(dependency),
      `${dependency} must stay independently owned`,
    );
  }

  assert.equal(
    fs.existsSync(path.join(root, "crates/vize_atelier_core/src/source_atlas.rs")),
    false,
  );
  assert.equal(fs.existsSync(path.join(root, "crates/vize_atelier_core/src/rendu.rs")), false);
  assert.equal(fs.existsSync(path.join(root, "crates/vize_atlas/src/lib.rs")), true);
  assert.equal(fs.existsSync(path.join(root, "crates/vize_rendu/src/lib.rs")), true);
  assert.equal(fs.existsSync(path.join(root, "crates/vize_flow/src/lib.rs")), true);
});

test("architecture docs distinguish Relief syntax from Croquis semantics", () => {
  const glossary = read("docs/content/architecture/source-atlas-glossary.md");
  assert.match(glossary, /Relief\s+\|\s+Owned\/source-faithful Vue-template syntax/i);
  assert.match(glossary, /Croquis\s+\|\s+Owned\/frontend-neutral semantic facts/i);
  assert.match(glossary, /Flow\s+\|\s+Frontend-neutral single-file control\/data\/effect graph/i);
  assert.match(glossary, /Rendu\s+\|\s+Owned\/frontend-neutral structured render HIR/i);
  assert.match(glossary, /Croquis CF.*cross-file module\/component aggregation/i);
});

test("Atlas is an open graph kernel, not a domain ledger", () => {
  const atlasLib = read("crates/vize_atlas/src/lib.rs");
  for (const rejected of [
    "SourceAtlasPlate",
    "SourceAtlasRoute",
    "SourceAtlasRegistry",
    "SourceAtlasCoordinate",
  ]) {
    assert.doesNotMatch(atlasLib, new RegExp(rejected));
  }
  for (const removedFile of [
    "coordinate.rs",
    "fallback.rs",
    "family.rs",
    "registry.rs",
    "report.rs",
    "route.rs",
  ]) {
    assert.equal(fs.existsSync(path.join(root, "crates/vize_atlas/src", removedFile)), false);
  }
  assert.match(read("crates/vize_atlas/src/provider.rs"), /fn supports\(/);
  assert.match(read("crates/vize_atlas/src/provider.rs"), /fn input_dependencies\(/);
  assert.match(read("crates/vize_atlas/src/provider.rs"), /fn dependency_requests\(/);
  assert.match(read("crates/vize_atlas/src/provider.rs"), /fn get_for_source</);
  assert.match(
    read("crates/vize_atlas/src/compilation/snapshot.rs"),
    /pub struct CompilationSnapshot/,
  );
  assert.match(read("crates/vize_atlas/src/compilation.rs"), /pub fn plan_requests/);
});

test("representation contracts stay independent of frontend producers", () => {
  const renduManifest = read("crates/vize_rendu/Cargo.toml");
  const flowManifest = read("crates/vize_flow/Cargo.toml");
  for (const forbidden of ["vize_relief", "vize_croquis", "vize_atelier_sfc", "vize_atelier_jsx"]) {
    assert.doesNotMatch(renduManifest, new RegExp(forbidden));
    assert.doesNotMatch(flowManifest, new RegExp(forbidden));
  }

  const croquisProduct = read("crates/vize_croquis/src/product.rs");
  const croquisModel = read("crates/vize_croquis/src/semantic.rs");
  const croquisSnapshot = read("crates/vize_croquis/src/semantic/types.rs");
  const croquisBuilder = read("crates/vize_croquis/src/semantic/builder.rs");
  assert.doesNotMatch(croquisProduct, /vize_relief|RootNode|ElementNode/);
  assert.doesNotMatch(croquisModel, /vize_relief|RootNode|ElementNode/);
  assert.doesNotMatch(croquisSnapshot, /vize_relief|RootNode|ElementNode/);
  assert.doesNotMatch(croquisBuilder, /vize_relief|RootNode|ElementNode/);

  const croquisManifest = read("crates/vize_croquis/Cargo.toml");
  assert.match(
    read("Cargo.toml"),
    /vize_croquis = \{ path = "crates\/vize_croquis", version = "=[^"]+", default-features = false \}/,
  );
  assert.match(croquisManifest, /default = \["relief-compat"\]/);
  assert.match(croquisManifest, /relief-compat = \["analysis", "dep:vize_relief"\]/);
  assert.match(croquisManifest, /vize_relief = \{ workspace = true, optional = true \}/);

  for (const consumer of ["vize_atelier_jsx", "vize_croquis_cf"]) {
    const manifest = read(`crates/${consumer}/Cargo.toml`);
    assert.match(
      manifest,
      /vize_croquis = \{ workspace = true, default-features = false, features = \["analysis"\] \}/,
      `${consumer} must consume Relief-free Croquis analysis`,
    );
  }
  assert.match(
    read("crates/vize_curator/Cargo.toml"),
    /vize_croquis = \{ workspace = true, default-features = false \}/,
  );

  const coreLib = read("crates/vize_atelier_core/src/lib.rs");
  assert.doesNotMatch(coreLib, /pub use vize_(atlas|rendu)/);

  const jsxProviders = read("crates/vize_atelier_jsx/src/atlas.rs");
  assert.match(jsxProviders, /type Product = FlowProduct/);
  assert.match(jsxProviders, /type Product = RenduProduct/);

  const projectProvider = read("crates/vize_croquis_cf/src/atlas.rs");
  assert.match(projectProvider, /fn dependency_requests/);
  assert.match(projectProvider, /get_for_source::<CroquisSemanticProduct>/);
});
