import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const REPORT_DIRECTORY = "tools/benchmarks/results/davinci";
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const harnessSourceWitnesses = new WeakSet();
const harnessSourcePins = new WeakSet();
const HARNESS_SOURCE_PATHS = Object.freeze([
  "Cargo.toml",
  "tools/benchmarks/crates/davinci_harness/Cargo.toml",
  "tools/benchmarks/crates/davinci_harness/src/lib.rs",
]);

function harnessSourceFile(root, relative) {
  const file = path.join(root, relative);
  const stat = fs.lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink(), "foreign harness source file");
  assert.equal(fs.realpathSync(file), file, "foreign harness source parent");
  const bytes = fs.readFileSync(file);
  return {
    path: relative,
    git_blob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    sha256: sha256(bytes),
  };
}

// Reuse the already retained exact-head Git listing before Cargo runs. These
// are before/after source observations, not compiler-open-time telemetry.
export function captureHarnessSource(root, wholeTree) {
  root = path.resolve(root);
  assert.equal(fs.realpathSync(root), root, "foreign harness source root");
  const lines = wholeTree.split("\n");
  const files = HARNESS_SOURCE_PATHS.map((relative) => {
    const matches = lines.filter((line) => line.endsWith(`\t${relative}`));
    assert.equal(matches.length, 1, "missing or duplicate harness Git source");
    const entry = /^100644 blob ([a-f0-9]{40})\t/.exec(matches[0]);
    assert.ok(entry, "wrong harness Git source mode or blob");
    assert.equal(
      matches[0],
      `100644 blob ${entry[1]}\t${relative}`,
      "malformed harness Git source entry",
    );
    const observed = harnessSourceFile(root, relative);
    assert.equal(observed.git_blob, entry[1], "harness source differs from pinned Git blob");
    return Object.freeze(observed);
  });
  const pin = Object.freeze({ root, files: Object.freeze(files) });
  harnessSourcePins.add(pin);
  return pin;
}

function validateHarnessSourcePin(root, pin) {
  assert.ok(harnessSourcePins.has(pin), "missing authenticated harness source pin");
  assert.equal(pin.root, root, "foreign harness source pin root");
  for (const original of pin.files)
    assert.deepEqual(harnessSourceFile(root, original.path), original, "harness source changed");
}

// Cargo owns version inheritance. Join its exact source metadata to the actual
// library compiler artifact; a copied or caller-written object is no witness.
export function bindHarnessSource(root, metadata, artifacts, sourcePin) {
  root = path.resolve(root);
  assert.equal(fs.realpathSync(root), root, "foreign harness source root");
  validateHarnessSourcePin(root, sourcePin);
  assert.equal(metadata.workspace_root, root, "foreign Cargo metadata workspace");
  assert.ok(Array.isArray(metadata.packages), "missing harness source packages");
  assert.ok(Array.isArray(artifacts), "missing harness build artifacts");
  const manifest = path.join(root, "tools/benchmarks/crates/davinci_harness/Cargo.toml");
  const source = path.join(path.dirname(manifest), "src/lib.rs");
  const packages = metadata.packages.filter((pkg) => pkg.name === "davinci_harness");
  assert.equal(packages.length, 1, "missing or duplicate source harness package");
  const pkg = packages[0];
  assert.equal(pkg.manifest_path, manifest, "foreign harness source manifest");
  assert.ok(
    typeof pkg.version === "string" && pkg.version.length,
    "missing harness source version",
  );
  assert.ok(
    typeof pkg.id === "string" && pkg.id.startsWith("path+file:"),
    "foreign harness package ID",
  );
  const packageURL = new URL(pkg.id.slice("path+".length));
  assert.equal(fileURLToPath(packageURL), path.dirname(manifest), "foreign harness package source");
  assert.equal(
    packageURL.hash,
    `#${pkg.version}`,
    "harness source version differs from package ID",
  );
  const libraries = artifacts.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.manifest_path === manifest &&
      Array.isArray(row.target?.kind) &&
      row.target.kind.includes("lib"),
  );
  assert.equal(libraries.length, 1, "missing or duplicate harness library artifact");
  assert.equal(
    libraries[0].package_id,
    pkg.id,
    "harness library package differs from source metadata",
  );
  assert.equal(libraries[0].target.src_path, source, "foreign harness library source");
  const witness = Object.freeze({
    root,
    version: pkg.version,
    package_id: pkg.id,
    manifest_path: manifest,
    manifest_sha256: sourcePin.files[1].sha256,
    workspace_manifest_sha256: sourcePin.files[0].sha256,
    library_source: source,
    library_source_sha256: sourcePin.files[2].sha256,
    source_pin: sourcePin,
  });
  harnessSourceWitnesses.add(witness);
  return witness;
}

export function validateHarnessReports(rows, witness) {
  assert.ok(harnessSourceWitnesses.has(witness), "missing authenticated harness source witness");
  validateHarnessSourcePin(witness.root, witness.source_pin);
  for (const [id, row] of rows) {
    assert.equal(
      row.file,
      path.join(witness.root, REPORT_DIRECTORY, `${id}.json`),
      `${id}: report differs from own source arm`,
    );
    assert.equal(
      row.report.harness_version,
      witness.version,
      `${id}: changed own-source harness_version`,
    );
  }
}
