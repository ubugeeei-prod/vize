import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import { sha256 } from "./support/bound-slot-attribute-oracle.mjs";

const dir = path.resolve(
  import.meta.dirname,
  "../_fixtures/differential/lint/deprecated-template-research-47c6",
);
const read = (file) => JSON.parse(fs.readFileSync(path.join(dir, file), "utf8"));

test("historical whole packets remain bound to their retained source custody", () => {
  const manifest = read("manifest.json");
  assert.equal(manifest.sourceRevision, "47c6f998d49cd6c7b790550098b3378639b949f7");
  for (const [file, expected] of Object.entries(manifest.files)) {
    const bytes = fs.readFileSync(path.join(dir, file));
    assert.equal(bytes.length, expected.bytes, file);
    assert.equal(sha256(bytes), expected.sha256, file);
  }
  const inventoryBytes = gunzipSync(fs.readFileSync(path.join(dir, "source-inventory.json.gz")));
  assert.equal(sha256(inventoryBytes), manifest.sourceInventoryDecompressedSha256);
  const inventory = JSON.parse(inventoryBytes.toString("utf8"));
  assert.equal(inventory.sourceRevision, manifest.sourceRevision);
  assert.equal(Object.keys(inventory.files).length, 10178);
  const receipt = read("observer-build-receipt.json");
  assert.equal(receipt.sourceRevision, manifest.sourceRevision);
  assert.equal(receipt.build.status, 0);
  assert.equal(sha256(fs.readFileSync(path.join(dir, "observer.rs"))), receipt.driver.sha256);
  assert.equal(
    receipt.files["source-files-before.json"].sha256,
    manifest.sourceInventoryDecompressedSha256,
  );
  assert.equal(
    receipt.files["source-files-after.json"].sha256,
    manifest.sourceInventoryDecompressedSha256,
  );
  assert.equal(
    receipt.binary.sha256,
    "e78028b4c31709037f85edf1d053f7d69974b2be4117ec788836eca3c3a6fdc7",
  );
  const execution = read("native-execution-receipt.json");
  assert.equal(execution.status, 0);
  assert.equal(execution.binarySha256, receipt.binary.sha256);
  assert.equal(
    execution.observerReceiptSha256,
    manifest.files["observer-build-receipt.json"].sha256,
  );
});

test("all 118 owned sources retain both entire observations without findings filtering", () => {
  const inputs = read("observer-input.json");
  const native = read("native-packets.json");
  const provider = fs
    .readFileSync(path.join(dir, "provider-packets.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.equal(inputs.length, 118);
  assert.equal(native.completedCases, 118);
  assert.equal(native.executionStatus, "completed");
  assert.equal(native.cases.length, 118);
  assert.equal(provider.length, 236);
  for (const input of inputs) {
    const sourceCase = native.cases.find((c) => c.id === input.id);
    assert.equal(sourceCase.source, input.source);
    assert.equal(sourceCase.filename, input.filename);
    assert.equal(sourceCase.observations.length, 2);
    assert.deepEqual(sourceCase.observations[0], sourceCase.observations[1]);
    const repeats = provider.filter((c) => c.id === input.id);
    assert.equal(repeats.length, 2);
    for (const capture of repeats) {
      assert.equal(capture.source, input.source);
      assert.equal(capture.sourceSha256, sha256(input.source));
      assert.equal(capture.providerError, undefined);
      assert.equal(capture.rawPackets.length, 1);
    }
    assert.deepEqual(repeats[0].rawPackets, repeats[1].rawPackets);
  }
  const summary = read("summary.json");
  assert.equal(summary.matrix.length, 10);
  assert.equal(summary.nativeRepeatDifferences, 0);
  assert.equal(summary.providerRepeatDifferences, 0);
  assert.equal(summary.providerErrors, 0);
  assert.equal(summary.providerFatalErrors, 0);
});
