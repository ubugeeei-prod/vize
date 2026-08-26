import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseToml } from "@iarna/toml";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const doctorRoot = path.join(repoRoot, "crates", "vize_doctor");
const scannedExtensions = new Set([".md", ".rs", ".toml"]);

type CargoDependency =
  | boolean
  | {
      package?: unknown;
      workspace?: unknown;
    };

type CargoManifest = {
  dependencies?: Record<string, CargoDependency>;
};

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

test("Doctor depends on the stage-named S0 alias", () => {
  const cargoToml = fs.readFileSync(path.join(doctorRoot, "Cargo.toml"), "utf8");
  const manifest = parseToml(cargoToml) as CargoManifest;
  const dependencies = manifest.dependencies ?? {};

  assert.deepEqual(dependencies.vize_s0, { workspace: true });
  assert.equal(dependencies.vize_carton, undefined);
});

test("Doctor does not name the Carton crate directly", () => {
  const offenders = [];

  for (const file of walkFiles(doctorRoot)) {
    const source = fs.readFileSync(file, "utf8");
    if (source.includes("vize_carton")) {
      offenders.push(path.relative(repoRoot, file));
    }
  }

  assert.deepEqual(offenders, []);
});
