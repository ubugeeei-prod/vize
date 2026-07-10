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
  assert.deepEqual(internalDependencies("vize_relief"), ["vize_carton"]);
  assert.deepEqual(internalDependencies("vize_croquis"), [
    "vize_armature",
    "vize_carton",
    "vize_relief",
  ]);
  assert.deepEqual(internalDependencies("vize_rendu"), [
    "vize_armature",
    "vize_atlas",
    "vize_carton",
    "vize_relief",
  ]);

  const atelierDependencies = internalDependencies("vize_atelier_core");
  for (const dependency of ["vize_atlas", "vize_croquis", "vize_relief", "vize_rendu"]) {
    assert.ok(
      atelierDependencies.includes(dependency),
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
});

test("architecture docs distinguish Relief syntax from Croquis semantics", () => {
  const glossary = read("docs/content/architecture/source-atlas-glossary.md");
  assert.match(
    glossary,
    /`Relief`\s+\| Source syntax: what node was written, its shape, and its location/i,
  );
  assert.match(
    glossary,
    /`Croquis`\s+\| Derived meaning: identity, scopes, bindings, usage, dependencies/i,
  );
  assert.match(glossary, /Transforming or normalizing a Relief node does not make it Croquis/);
  assert.match(glossary, /`vize_atelier_core` \| Shared transforms and JavaScript emission/);
});
