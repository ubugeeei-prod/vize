import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// Offline shape guard for the JSX/TSX ecosystem reference manifest (the manual
// ecosystem-coverage workflow for #1491). The smoke that actually clones and
// compiles these repos lives in the ignored Rust test
// `crates/vize_atelier_jsx/tests/ecosystem_smoke.rs`; this test only validates
// that the committed manifest stays pinned and well-formed, so PR CI keeps the
// ecosystem list honest without any network access.

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath = path.join(
  root,
  "crates",
  "vize_atelier_jsx",
  "tests",
  "ecosystem",
  "testbeds.json",
);

interface ManifestEntry {
  id: string;
  displayName: string;
  repository: string;
  revision: string;
  branch: string;
  roots: string[];
  extensions: string[];
  kind: string;
}

interface Manifest {
  schemaVersion: number;
  description: string;
  command: string;
  references: ManifestEntry[];
  testbeds: ManifestEntry[];
}

function readManifest(): Manifest {
  return JSON.parse(fs.readFileSync(manifestPath, "utf8")) as Manifest;
}

const requiredReferences = ["babel-plugin-jsx", "vue-jsx-vapor"] as const;

test("JSX ecosystem manifest pins the requested reference suites", () => {
  const manifest = readManifest();
  assert.equal(manifest.schemaVersion, 1);

  const referenceIds = new Set(manifest.references.map((entry) => entry.id));
  for (const id of requiredReferences) {
    assert.ok(referenceIds.has(id), `${id} should be a pinned reference suite`);
  }
});

test("real-world component libraries stay in the Vize-wide fixture registry", () => {
  const manifest = readManifest();
  assert.deepEqual(manifest.testbeds, []);
});

test("every JSX ecosystem entry pins a full-SHA revision, github repo, roots, and extensions", () => {
  const manifest = readManifest();

  for (const entry of [...manifest.references, ...manifest.testbeds]) {
    assert.match(
      entry.revision,
      /^[0-9a-f]{40}$/,
      `${entry.id} should pin a full commit SHA (got "${entry.revision}")`,
    );
    assert.match(
      entry.repository,
      /^https:\/\/github\.com\/.+\.git$/,
      `${entry.id} should declare an https github repository`,
    );
    assert.ok(entry.roots.length > 0, `${entry.id} should declare at least one walk root`);
    assert.ok(
      entry.extensions.length > 0 && entry.extensions.every((ext) => ext.startsWith(".")),
      `${entry.id} should declare dotted file extensions`,
    );
    assert.ok(entry.displayName.length > 0, `${entry.id} should declare a display name`);
    assert.ok(entry.branch.length > 0, `${entry.id} should record the upstream branch it pins`);
  }
});

test("JSX ecosystem manifest documents the manual run command", () => {
  const manifest = readManifest();
  assert.match(
    manifest.command,
    /cargo test -p vize_atelier_jsx --test ecosystem_smoke -- --ignored/,
    "manifest should document how to run the manual ecosystem smoke",
  );
  assert.ok(manifest.description.length > 0, "manifest should describe the manual workflow");
});
