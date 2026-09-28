import assert from "node:assert/strict";
import path from "node:path";
import { loadProductManifest, readPinnedArtifact } from "./harness.mjs";

export const FORMATTER_ARGV = ["fmt", "--no-config", "--write", "App.vue"];
export const CONFIGURED_FORMATTER_ARGV = [
  "fmt",
  "--config",
  "vize.config.json",
  "--write",
  "App.vue",
];
export { sha256 } from "./harness.mjs";

// Deliberately only the first executable product. Other real adapters stack later.
export function loadFormatterManifest(manifestPath) {
  const loaded = loadProductManifest(manifestPath, "formatter");
  const { manifest } = loaded;
  assert.deepEqual(manifest.adapterOptions, { argv: FORMATTER_ARGV, passes: 3 });
  assert(Array.isArray(manifest.cases) && manifest.cases.length > 0, "planned cases are required");
  const root = path.dirname(manifestPath);
  readPinnedArtifact(root, manifest.configurationReference);
  assert.equal(manifest.baselineCapture.kind, "release-candidate-observation");
  assert.match(manifest.baselineCapture.baselineRevision, /^[a-f0-9]{40}$/);
  assert.equal(
    manifest.baselineCapture.artifacts.length,
    2,
    "candidate capture and artifact receipt are required",
  );
  for (const capture of manifest.baselineCapture.artifacts) readPinnedArtifact(root, capture);
  const cases = manifest.cases.map((fixture) => {
    assert.match(fixture.id, /^formatter\/[a-z0-9/-]+$/);
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
    const entry = fixture.inputs.files[0];
    assert(["App.vue", "App.vue.txt"].includes(entry.path), "unsupported formatter input");
    if (entry.path === "App.vue.txt") {
      assert.equal(entry.kind, "vue");
      assert.equal(entry.runtimeFileName, "App.vue");
    }
    assert(Array.isArray(fixture.inputs.config) && fixture.inputs.config.length <= 1);
    assert(!path.isAbsolute(fixture.inputs.root), "input root must be relative");
    const inputFile = {
      ...entry,
      path: path.join(fixture.inputs.root, entry.path),
    };
    const config = fixture.inputs.config.map((file) => {
      assert.equal(file.path, "vize.config.json", "only the explicit JSON config is supported");
      const bytes = readPinnedArtifact(root, {
        ...file,
        path: path.join(fixture.inputs.root, file.path),
      });
      const options = JSON.parse(bytes.toString("utf8"));
      assert(["2", "2.7", "3"].includes(options.vue?.version), "explicit Vue version required");
      return { path: file.path, sha256: file.sha256, bytes };
    });
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
      input: readPinnedArtifact(root, inputFile),
      expected: readPinnedArtifact(root, expectedFiles[0]),
      config,
      argv: [...(config.length ? CONFIGURED_FORMATTER_ARGV : FORMATTER_ARGV)],
    };
  });
  return { ...loaded, cases };
}
