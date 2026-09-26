import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const FORMATTER_ARGV = ["fmt", "--no-config", "--write", "App.vue"];
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

function artifact(root, file) {
  assert.equal(typeof file.path, "string", "artifact path is required");
  assert(!path.isAbsolute(file.path), "artifact path must be relative");
  const resolved = fs.realpathSync(path.resolve(root, file.path));
  const relative = path.relative(fs.realpathSync(root), resolved);
  assert(
    relative !== "" && !relative.startsWith("..") && !path.isAbsolute(relative),
    "artifact must remain inside its fixture root",
  );
  assert(fs.statSync(resolved).isFile(), "artifact must be a file");
  assert.match(file.sha256, /^[a-f0-9]{64}$/, "artifact sha256 is required");
  const bytes = fs.readFileSync(resolved);
  assert.equal(sha256(bytes), file.sha256, `immutable artifact SHA256 mismatch: ${file.path}`);
  return bytes;
}

// Deliberately only the first executable product. Other real adapters stack later.
export function loadFormatterManifest(manifestPath) {
  const raw = fs.readFileSync(manifestPath);
  const manifest = JSON.parse(raw.toString("utf8"));
  assert.equal(manifest.schema, "vize.differential.manifest");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.product, "formatter", "no registered adapter for this product");
  assert.match(manifest.baseRevision, /^[a-f0-9]{40}$/);
  assert.deepEqual(manifest.adapterOptions, { argv: FORMATTER_ARGV, passes: 3 });
  assert(Array.isArray(manifest.cases) && manifest.cases.length > 0, "planned cases are required");
  const root = path.dirname(manifestPath);
  artifact(root, manifest.configurationReference);
  assert.equal(manifest.baselineCapture.kind, "release-candidate-observation");
  assert.match(manifest.baselineCapture.baselineRevision, /^[a-f0-9]{40}$/);
  assert.equal(
    manifest.baselineCapture.artifacts.length,
    2,
    "candidate capture and artifact receipt are required",
  );
  for (const capture of manifest.baselineCapture.artifacts) artifact(root, capture);
  const ids = new Set();
  const cases = manifest.cases.map((fixture) => {
    assert.match(fixture.id, /^formatter\/[a-z0-9/-]+$/);
    assert(!ids.has(fixture.id), `duplicate planned case: ${fixture.id}`);
    ids.add(fixture.id);
    assert(["draft", "active"].includes(fixture.state));
    assert.deepEqual(fixture.targets, ["fmt"]);
    assert.equal(fixture.adapters.legacy, "formatter-cli-v1");
    assert.equal(fixture.adapters.native, null, "native formatter adapter is not registered");
    assert.equal(typeof fixture.adapters.reasons.native, "string");
    assert(fixture.adapters.reasons.native.length > 0);
    assert.equal(fixture.comparison.contract, "formatter-byte-output-v1");
    assert.deepEqual(fixture.comparison.requiredFacets, ["full-output-bytes", "idempotence"]);
    assert.deepEqual(fixture.comparison.transportMappings, []);
    assert(["pending", "captured"].includes(fixture.expectations.legacy.state));
    if (fixture.expectations.legacy.state === "captured") {
      assert.equal(
        fixture.expectations.legacy.baselineRevision,
        manifest.baselineCapture.baselineRevision,
      );
    }
    assert.equal(fixture.inputs.files.length, 1);
    assert.equal(fixture.inputs.files[0].path, "App.vue");
    assert.deepEqual(fixture.inputs.config, []);
    assert(!path.isAbsolute(fixture.inputs.root), "input root must be relative");
    const inputFile = {
      ...fixture.inputs.files[0],
      path: path.join(fixture.inputs.root, "App.vue"),
    };
    const expectedFiles = fixture.expectations.legacy.artifacts;
    assert.equal(expectedFiles.length, 1);
    assert(
      !expectedFiles[0].path.endsWith(".vue"),
      "output artifact must not join the authored corpus",
    );
    assert(Array.isArray(fixture.provenance.commits) && fixture.provenance.commits.length > 0);
    for (const origin of fixture.provenance.commits) assert.match(origin.sha, /^[a-f0-9]{40}$/);
    assert(Array.isArray(fixture.witnesses) && fixture.witnesses.length > 0);
    return {
      ...fixture,
      input: artifact(root, inputFile),
      expected: artifact(root, expectedFiles[0]),
    };
  });
  return { manifest, cases, manifestSha256: sha256(raw) };
}
