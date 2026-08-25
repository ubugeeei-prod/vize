import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const glyphRoot = path.join(repoRoot, "crates", "vize_glyph");
const scannedExtensions = new Set([".md", ".rs", ".toml"]);

function* walkFiles(dir: string): Generator<string> {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkFiles(fullPath);
    } else if (entry.isFile() && scannedExtensions.has(path.extname(entry.name))) {
      yield fullPath;
    }
  }
}

test("Glyph depends on the stage-named S0 alias", () => {
  const cargoToml = fs.readFileSync(path.join(glyphRoot, "Cargo.toml"), "utf8");

  assert.match(cargoToml, /^vize_s0\.workspace = true$/m);
  assert.doesNotMatch(cargoToml, /^vize_carton\.workspace = true$/m);
});

test("Glyph does not name the Carton crate directly", () => {
  const offenders = [];

  for (const file of walkFiles(glyphRoot)) {
    const source = fs.readFileSync(file, "utf8");
    if (source.includes("vize_carton")) {
      offenders.push(path.relative(repoRoot, file));
    }
  }

  assert.deepEqual(offenders, []);
});
