import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import {
  sourceSessionUnsupportedLegacy,
  SOURCE_SESSION_UNSUPPORTED_REASON,
} from "../differential/lsp-source-sessions.ts";
import { validateNativeCapacitySourceSession } from "../differential/lsp-native-capacity-session.ts";
import {
  NATIVE_CAPACITY_ID,
  NATIVE_CAPACITY_PLAN,
  NATIVE_CAPACITY_UNSUPPORTED_REASON,
} from "../differential/lsp-native-capacity-contract.ts";
import type { FixtureData } from "../differential/lsp-types.ts";

const manifestPath = fileURLToPath(
  new URL("../_fixtures/differential/lsp/manifest.json", import.meta.url),
);
const loaded = loadLspManifest(manifestPath);
const found = loaded.cases.find((row) => row.id === NATIVE_CAPACITY_ID);
assert.ok(found);
const fixture = found;
const files = new Map(fixture.files.map((file) => [file.runtimePath, file.bytes] as const));
const plan = JSON.parse(
  readPinnedArtifact(path.resolve(path.dirname(manifestPath), fixture.inputs.root), {
    path: fixture.data.expected.source,
    sha256: fixture.data.expected.sha256,
  }).toString("utf8"),
) as Record<string, unknown>;

test("capacity registration retains whole source/wire custody and claims zero generic native parity", () => {
  assert.equal(fixture.requestCount, 0);
  assert.deepEqual(fixture.requests, []);
  assert.equal(fixture.files.length, 15);
  assert.deepEqual(plan, NATIVE_CAPACITY_PLAN);
  assert.deepEqual(sourceSessionUnsupportedLegacy(fixture.data.sourceSession), {
    state: "unsupported",
    verdict: "unsupported-transition",
    comparator: "source-inspected-session",
    reason: NATIVE_CAPACITY_UNSUPPORTED_REASON,
  });
  assert.equal(sourceSessionUnsupportedLegacy().reason, SOURCE_SESSION_UNSUPPORTED_REASON);
  assert.equal(loaded.cases.filter((row) => row.id === NATIVE_CAPACITY_ID).length, 1);
  const ids = loaded.manifest.cases.map((row) => row.id);
  assert.deepEqual(
    ids,
    [...ids].sort((a, b) => a.localeCompare(b)),
  );
});

test("capacity registration rejects unknown kinds, fields and shortened owner inventory", () => {
  for (const candidate of [
    { ...fixture.data, id: "lsp/source-session/unregistered" },
    { ...fixture.data, sourceSession: { ...fixture.data.sourceSession, kind: "unknown" } },
    { ...fixture.data, sourceSession: { ...fixture.data.sourceSession, bulkDocuments: 128 } },
    { ...fixture.data, unexpected: true },
    { ...fixture.data, files: [] },
    { ...fixture.data, expected: { source: "response.expected.json", sha256: "0".repeat(64) } },
  ])
    assert.throws(() => validateNativeCapacitySourceSession(candidate as FixtureData, files, plan));
  assert.throws(() =>
    validateNativeCapacitySourceSession(fixture.data, files, {
      ...plan,
      genericNativeParityChecked: 129,
    }),
  );
});

test("capacity registration rejects missing source pairs and modified full wire", () => {
  const missing = new Map(files);
  missing.delete("protocol/project-input.json.gz");
  assert.throws(() => validateNativeCapacitySourceSession(fixture.data, missing, plan));
  const changed = new Map(files);
  const original = changed.get("custody/after-client.bin.gz");
  assert.ok(original);
  const bytes = Buffer.from(original);
  bytes.writeUInt8(bytes.readUInt8(20) ^ 1, 20);
  changed.set("custody/after-client.bin.gz", bytes);
  assert.throws(() => validateNativeCapacitySourceSession(fixture.data, changed, plan));
});
