import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import {
  originalAsciiCaller,
  qualifyAsciiOriginalCallers,
  retainedAsciiCallerFrame,
} from "./support/sdk-ascii-symbol-original-callers.ts";

const manifest = { originalSource: "5c54b1195fa8c0bf151e0a2862bf4238a3597b4e" };

void test("sealed whole SDK reader and original frames retain historical bodies and refuse partial custody", () => {
  const entries = retainedAsciiCallerFrame("sdk");
  const result = qualifyAsciiOriginalCallers(entries);
  for (const [kind, source, tree] of [
    [
      "original",
      "5c54b1195fa8c0bf151e0a2862bf4238a3597b4e",
      "15a32b70bab4deaa3e6156c32075587f71796fb3",
    ],
    ["sdk", "8635d85f7e511454f9a3d5b8553db6c237369def", "d868dba67c7820fc598d819e84b83a1ca8df0337"],
    [
      "reader",
      "6354e9b92ef842816f8f8be0d104b067f4175e96",
      "134d18f3a0e751513c12c2f56165fcdc86ec3304",
    ],
  ]) {
    const frame = retainedAsciiCallerFrame(kind);
    assert.deepEqual(qualifyAsciiOriginalCallers(frame), {
      authority: "complete-ascii-original-caller-source-frame",
      kind,
      source,
      tree,
      originalSource: manifest.originalSource,
      wholeCallerBodies: 10,
      runtimeCredit: false,
      performanceCredit: false,
    });
  }
  const reader = retainedAsciiCallerFrame("reader");
  const sdkRequest = retainedAsciiCallerFrame("sdk").find(({ file }) =>
    file.endsWith("warm-type-backed-requests.ts"),
  );
  assert.ok(sdkRequest);
  assert.throws(() =>
    qualifyAsciiOriginalCallers(
      reader.map((entry) => (entry.file === sdkRequest.file ? sdkRequest : entry)),
    ),
  );
  const first = originalAsciiCaller(manifest.originalSource, entries[0].file, entries);
  const same = Buffer.from(first);
  first[0] ^= 1;
  assert.deepEqual(originalAsciiCaller(manifest.originalSource, entries[0].file, entries), same);
  const variant = retainedAsciiCallerFrame("reader");
  variant[0].body[0] ^= 1;
  assert.deepEqual(retainedAsciiCallerFrame("reader"), reader);
  assert.throws(() => retainedAsciiCallerFrame("unknown"));
  assert.equal(result.wholeCallerBodies, 10);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
  for (const entry of entries) {
    assert.throws(() =>
      qualifyAsciiOriginalCallers(entries.filter(({ file }) => file !== entry.file)),
    );
    assert.throws(() => qualifyAsciiOriginalCallers([...entries, entry]));
    for (const mode of ["100755", "120000"])
      assert.throws(() =>
        qualifyAsciiOriginalCallers(
          entries.map((item) => (item.file === entry.file ? { ...item, mode } : item)),
        ),
      );
    const body = Buffer.from(entry.body);
    body[0] ^= 1;
    const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
    assert.throws(() =>
      qualifyAsciiOriginalCallers(
        entries.map((item) => (item.file === entry.file ? { ...item, body, blob } : item)),
      ),
    );
  }
  assert.throws(() =>
    qualifyAsciiOriginalCallers(
      entries.map((entry, index) => (index === 0 ? { ...entry, file: "unknown.ts" } : entry)),
    ),
  );
  assert.throws(() => originalAsciiCaller("f".repeat(40), entries[0].file, entries));
  assert.throws(() => originalAsciiCaller(manifest.originalSource, "unknown.ts", entries));
});

void test("the actual owned SDK/Reader whole frame remains distinct from every historical frame", () => {
  const composed = retainedAsciiCallerFrame("owned-sdk-reader");
  const receipt = qualifyAsciiOriginalCallers(composed);
  assert.equal(receipt.kind, "owned-sdk-reader");
  assert.equal(receipt.source, "fbdaa063d15302b8f2ff8693f57336cba007d056");
  assert.equal(receipt.tree, "160337c9ffa80e18ae3d7b29b3d22a253cf3f290");
  assert.equal(receipt.wholeCallerBodies, 10);
  assert.equal(receipt.runtimeCredit, false);
  for (const kind of ["original", "sdk", "reader"]) {
    const historical = retainedAsciiCallerFrame(kind);
    const workflow = historical.find((entry) =>
      entry.file.endsWith("warm-type-backed-workflow.ts"),
    );
    assert.ok(workflow);
    assert.throws(() =>
      qualifyAsciiOriginalCallers(
        composed.map((entry) => (entry.file === workflow.file ? workflow : entry)),
      ),
    );
  }
});

void test("physical and raw current callers select the retained fourth SDK frame", async () => {
  const { default: path } = await import("node:path");
  const { authenticateSdkComponentJoin } = await import("./support/sdk-component-join.mjs");
  const receipt = authenticateSdkComponentJoin(path.resolve(import.meta.dirname, "../.."));
  assert.equal(receipt.actualSdkAsciiBodies, 10);
  assert.equal(receipt.currentSdkBodies, 112);
  assert.equal(receipt.runtimeCredit, false);
  assert.equal(receipt.performanceCredit, false);
});
